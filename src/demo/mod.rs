use anyhow::Result;
use lib::prelude::*;
use parking_lot::{Condvar, Mutex};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, VecDeque},
    ops::RangeBounds,
    sync::Arc,
};

mod format;
pub use format::{Data, Demo, Event, Metadata};

mod stage;
pub use stage::{Stage, Stages};

mod audio;
use audio::Stream;

mod midi;

#[cfg(test)]
mod audio_test;

pub struct Player {
    stages: Option<Stages>,
    stream: Arc<Stream>,

    playing: bool,
    t: f32,
    rms: f32,
    next_stage: Option<&'static str>,

    meta: Metadata,
    events: Vec<(f32, Event)>,
    events_i: usize,
    data: Vec<(f32, Data)>,
    data_i: usize,
}

impl Player {
    pub fn new(
        file: &str,
        t0: f32,
        stage0: &'static str,
        stages: HashMap<&'static str, Box<dyn Stage + Send>>,
    ) -> Result<Self> {
        let Demo {
            meta,
            vorbis,
            events,
            data,
        } = Demo::load_bytes(&lib::resource::read(file))?;

        let stream = Arc::new(Stream::new(meta, vorbis, t0)?);

        Ok(Self {
            stages: Some(Stages::new(stage0, stages)),
            stream,

            playing: false,
            t: t0,
            rms: 0.0,
            next_stage: None,

            meta,
            events,
            events_i: 0,
            data,
            data_i: 0,
        })
    }

    pub async fn key(&mut self, state: KeyState, key: Key) {
        self.stages = Some(self.stages.take().unwrap().key(self, state, key).await);
    }

    pub async fn update(&mut self, dt: f32) {
        if let Some(next) = self.next_stage.take() {
            self.stages = Some(self.stages.take().unwrap().go(self, next).await);
        }

        let t = self.stream.t();
        let mut events = SmallVec::<[(f32, Event); 8]>::new();

        if self.playing {
            // Update data stream
            while self.data_i < self.data.len() && self.data[self.data_i].0 <= t {
                self.rms = self.data[self.data_i].1.rms;
                self.data_i += 1;
            }

            // Update event stream
            while self.events_i < self.events.len() && self.events[self.events_i].0 <= t {
                events.push(self.events[self.events_i]);
                self.events_i += 1;
            }

            self.t += dt;
        }

        // Dispatch events
        for (et, ev) in events.into_iter() {
            log::debug!("{:?} et={}, t={}, self.t={}, delta={}", ev, et, t, self.t, self.t - t);
            self.stages = Some(self.stages.take().unwrap().event(self, ev).await);

            if let Some(next) = self.next_stage.take() {
                self.stages = Some(self.stages.take().unwrap().go(self, next).await);
            }
        }

        self.stages = Some(self.stages.take().unwrap().update(self, dt).await);
    }

    pub fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.stages = Some(self.stages.take().unwrap().view(frame, view));
    }

    pub fn play(&mut self) {
        if !self.playing {
            self.playing = true;
            self.stream.play();
        }
    }

    pub async fn trigger(&mut self, ev: Event) {
        // log::debug!("Trigger: {:?} t={}", ev, self.t);
        self.stages = Some(self.stages.take().unwrap().event(self, ev).await);
    }

    pub async fn go(&mut self, to: &'static str) {
        self.next_stage = Some(to);
    }

    pub fn events<'a>(
        &'a self,
        time_range: impl RangeBounds<f32> + 'a,
    ) -> impl Iterator<Item = (f32, Event)> + 'a {
        // Binary search to find the first event in the range
        let first = self
            .events
            .partition_point(|(t, _)| !time_range.contains(t));

        // Iterate until we get to an event no longer in the range
        self.events[first..]
            .iter()
            .cloned()
            .take_while(move |(t, _)| time_range.contains(t))
    }

    pub fn t(&self) -> f32 {
        self.stream.t()
    }

    pub fn rms(&self) -> f32 {
        self.rms
    }
}
