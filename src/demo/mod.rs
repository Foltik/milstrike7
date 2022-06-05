use std::collections::VecDeque;
use parking_lot::{Condvar, Mutex};
use anyhow::Result;

mod format; pub use format::{Demo, Metadata, Event, Data};
mod audio; use audio::Stream;
mod midi;

#[cfg(test)]
mod audio_test;

pub struct Player {
    meta: Metadata,

    stream: Stream,
    events: Vec<(f32, Event)>,
    data: Vec<(f32, Data)>,

    playing: bool,
    t: f32,
    rms: f32,

    data_idx: usize,
    events_idx: usize,
    events_buffer: VecDeque<Event>,
}

impl Player {
    pub fn new(file: &str, start: f32) -> Result<Self> {
        let Demo { meta, audio, events, data } = Demo::load_bytes(&lib::resource::read(file))?;

        let stream = Stream::new(meta, audio, start)?;

        Ok(Self {
            meta,

            stream,
            events,
            data,

            playing: false,
            t: start,
            rms: 0.0,

            data_idx: 0,
            events_idx: 0,
            events_buffer: VecDeque::new(),
        })
    }

    pub fn update(&mut self, dt: f32) {
        if self.playing {
            // Update data stream
            while self.data_idx < self.data.len() && self.data[self.data_idx].0 <= self.t {
                self.rms = self.data[self.data_idx].1.rms;
                self.data_idx += 1;
            }

            // Update event stream
            while self.events_idx < self.events.len() && self.events[self.events_idx].0 <= self.t {
                self.events_buffer.push_back(self.events[self.events_idx].1);
                self.events_idx += 1;
            }

            self.t = self.t + dt;
        }
    }

    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events_buffer.drain(..)
    }

    pub fn events_after(&self, t: f32) -> impl Iterator<Item = (f32, Event)> + '_ {
        let idx = self.events.partition_point(|(et, _)| *et >= t);
        self.events[idx..].iter().cloned()
    }

    pub fn events_range(&self, start: f32, end: f32) -> impl Iterator<Item = (f32, Event)> + '_ {
        self.events_after(start).take_while(move |(t, _)| *t <= end)
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn rms(&self) -> f32 {
        self.rms
    }

    pub fn play(&mut self) {
        if !self.playing {
            self.playing = true;
            self.stream.play();
        }
    }
}
