use async_trait::async_trait;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::{IsoTri, IsoTriPass};

pub struct Test1 {
    strobe: Decay,
    beat: Decay,
    t: f32,
    t_mul: f32,

    tri: IsoTriPass,
}

impl Test1 {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            strobe: Decay::new(25.0),
            beat: Decay::new(200.0),
            t: 0.0,
            t_mul: 1.0,

            tri: IsoTriPass::new(
                device,
                IsoTri {
                    color: [1.0, 0.5, 0.0],
                    aspect: 9.0 / 16.0,
                    t: 0.0,
                    r: 0.0,
                    weight: 0.5,
                    thickness: 0.5,
                },
            ),
        }
    }
}

#[async_trait]
impl Stage for Test1 {
    async fn init(&mut self, p: &mut Player) {}

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 250.0 * p.rms() * self.t_mul * dt;

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

    async fn event(&mut self, p: &mut Player, ev: Event) {
        match ev {
            Event::Snare => self.strobe.set(),
            Event::Kick => self.beat.set(),
            Event::Strobe => self.t_mul = 3.0,
            _ => {}
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            Key::Return => p.go("test2").await,
            _ => {}
        }
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.tri.encode(frame, view);
    }
}
