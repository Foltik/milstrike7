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

mod scene;
pub use scene::{Scene, Scenes};

mod audio;
use audio::Stream;

mod midi;

#[cfg(test)]
mod audio_test;

pub struct Player {
    scenes: Option<Scenes>,
    stream: Arc<Stream>,

    playing: bool,
    t: f32,
    rms: f32,
    next_scene: Option<&'static str>,

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
        scene0: &'static str,
        scenes: HashMap<&'static str, Box<dyn Scene + Send>>,
    ) -> Result<Self> {
        let Demo {
            meta,
            audio,
            events,
            data,
        } = Demo::load_bytes(&lib::resource::read(file))?;

        let stream = Arc::new(Stream::new(meta, audio, t0)?);

        Ok(Self {
            scenes: Some(Scenes::new(scene0, scenes)),
            stream,

            playing: false,
            t: t0,
            rms: 0.0,
            next_scene: None,

            meta,
            events,
            events_i: 0,
            data,
            data_i: 0,
        })
    }

    pub async fn key(&mut self, state: KeyState, key: Key) {
        self.scenes = Some(self.scenes.take().unwrap().key(self, state, key).await);
    }

    pub async fn update(&mut self, dt: f32) {
        if let Some(next) = self.next_scene.take() {
            self.scenes = Some(self.scenes.take().unwrap().go(self, next).await);
        }

        let mut events = SmallVec::<[Event; 8]>::new();

        if self.playing {
            // Update data stream
            while self.data_i < self.data.len() && self.data[self.data_i].0 <= self.t {
                self.rms = self.data[self.data_i].1.rms;
                self.data_i += 1;
            }

            // Update event stream
            while self.events_i < self.events.len() && self.events[self.events_i].0 <= self.t {
                events.push(self.events[self.events_i].1);
                self.events_i += 1;
            }

            self.t = self.t + dt;
        }

        // Dispatch events
        for ev in events.into_iter().rev() {
            log::debug!("Event: {:?}", ev);
            self.scenes = Some(self.scenes.take().unwrap().event(self, ev).await);
        }

        self.scenes = Some(self.scenes.take().unwrap().update(self, dt).await);
    }

    pub fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.scenes = Some(self.scenes.take().unwrap().view(frame, view));
    }

    pub fn play(&mut self) {
        if !self.playing {
            self.playing = true;
            self.stream.play();
        }
    }

    pub async fn go(&mut self, to: &'static str) {
        self.next_scene = Some(to);
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
        self.t
    }

    pub fn rms(&self) -> f32 {
        self.rms
    }
}
