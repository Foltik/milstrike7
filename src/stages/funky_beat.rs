use async_trait::async_trait;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct FunkyBeat {
    segment: Segment,
    t: f32,
    t_mul: f32,
}

enum Segment {
    Init,
}

impl FunkyBeat {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            segment: Segment::Init,
            t: 0.0,
            t_mul: 1.0,
        }
    }
}

#[async_trait]
impl Stage for FunkyBeat {
    async fn init(&mut self, p: &mut Player) {}

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 250.0 * p.rms() * self.t_mul * dt;
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        match ev {
            _ => {}
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            _ => {}
        }
    }

    fn view(&mut self, frame: &mut Frame, depth: &wgpu::RawTextureView, view: &wgpu::RawTextureView) {
        match self.segment {
            Segment::Init => {},
        }
    }
}
