use lib::prelude::*;
use async_trait::async_trait;

use crate::Model;
use crate::dispatch::Render;
use crate::pipeline::{IsoTriPass, IsoTri};
use crate::demo::Event;

pub struct Test {
    strobe: Decay,
    beat: Decay,
    t: f32,
    t_mul: f32,

    tri: IsoTriPass,
}

impl Test {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            strobe: Decay::new(25.0),
            beat: Decay::new(200.0),
            t: 0.0,
            t_mul: 1.0,

            tri: IsoTriPass::new(device, IsoTri {
                color: [1.0, 0.5, 0.0],
                aspect: 9.0 / 16.0,
                t: 0.0,
                r: 0.0,
                weight: 0.5,
                thickness: 0.5,
            }),
        }
    }
}

#[async_trait]
impl Render for Test {
    async fn init(&mut self, _app: &App, m: &mut Model) {}

    async fn update(&mut self, _app: &App, m: &mut Model, dt: f32) {
        self.t += 250.0 * m.rms() * self.t_mul * dt;

        self.tri.update(self.t);
        self.beat.update(dt);
        self.strobe.update(dt);

        if self.beat.off() {
            self.tri.thickness = 0.5 + 0.5 * self.strobe.v();
        } else {
            self.tri.r = self.beat.v();
            self.tri.thickness = 0.5 + self.beat.v();
            self.tri.weight = 0.5 + 0.5 * self.beat.v();
        }
    }

    async fn midi(&mut self, _app: &App, m: &mut Model, ev: Event) {
        match ev {
            Event::Snare => self.strobe.set(),
            Event::Kick => self.beat.set(),
            Event::Strobe => self.t_mul = 3.0,
            _ => {},
        }
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.tri.encode(frame, view);
    }
}
