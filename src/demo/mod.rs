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

    event_buffer: VecDeque<Event>,
}

impl Player {
    pub fn new(file: &str, start: f32) -> Result<Self> {
        let Demo { meta, audio, mut events, mut data } = Demo::load_bytes(&lib::resource::read(file))?;
        events.reverse();
        data.reverse();

        let stream = Stream::new(meta, audio, start)?;

        Ok(Self {
            meta,

            stream,
            events,
            data,

            playing: false,
            t: start,
            rms: 0.0,

            event_buffer: VecDeque::new(),
        })
    }

    pub fn update(&mut self, dt: f32) {
        if self.playing {
            while self.data.len() != 0 && self.data.last().unwrap().0 <= self.t {
                self.rms = self.data.pop().unwrap().1.rms;
            }

            while self.events.len() != 0 && self.events.last().unwrap().0 <= self.t {
                self.event_buffer.push_back(self.events.pop().unwrap().1);
            }

            self.t = self.t + dt;
        }
    }

    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.event_buffer.drain(..)
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
