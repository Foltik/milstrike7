use lib::prelude::*;
use async_trait::async_trait;

use crate::pipeline::{IsoTriPass, IsoTri};
use crate::demo::{Event, Player, Stage};

pub struct TestSegments {
    hat: Decay,
    kick: Decay,
    t: f32,
    t_mul: f32,

    segment: Segment,

    tri1: IsoTriPass,
    tri2: IsoTriPass,
}

enum Segment {
    Tri1,
    Tri2,
}

impl TestSegments {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            hat: Decay::new(25.0),
            kick: Decay::new(200.0),
            t: 0.0,
            t_mul: 1.0,

            segment: Segment::Tri1,

            tri1: IsoTriPass::new(device, IsoTri {
                color: [1.0, 0.5, 0.0],
                aspect: 9.0 / 16.0,
                t: 0.0,
                r: 0.0,
                weight: 0.5,
                thickness: 0.5,
            }),
            tri2: IsoTriPass::new(device, IsoTri {
                color: [0.6, 0.0, 0.8],
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
impl Stage for TestSegments {
    async fn init(&mut self, p: &mut Player) {}

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 250.0 * p.rms() * self.t_mul * dt;

        let tri = match self.segment {
            Segment::Tri1 => &mut self.tri1,
            Segment::Tri2 => &mut self.tri2,
        };

        tri.update(self.t);
        self.kick.update(dt);
        self.hat.update(dt);

        if self.kick.off() {
            tri.r = 0.0;
            tri.thickness = 0.5 + 0.5 * self.hat.v();
            tri.weight = 0.5;
        } else {
            tri.r = self.kick.v();
            tri.thickness = 0.5 + self.kick.v();
            tri.weight = 0.5 + 0.5 * self.kick.v();
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        match ev {
            Event::Beat { id: 61, t } => self.hat.set_t(t),
            Event::Beat { id: 60, t } => self.kick.set_t(t),
            // Event::Trigger { id: 63 } => self.segment = match self.segment {
            //     Segment::Tri1 => Segment::Tri2,
            //     Segment::Tri2 => Segment::Tri1,
            // },

            // Event::Strobe => self.t_mul = 3.0,
            _ => {},
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            Key::Return => match self.segment {
                Segment::Tri1 => self.segment = Segment::Tri2,
                Segment::Tri2 => self.segment = Segment::Tri1,
            },
            _ => {}
        }
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        match self.segment {
            Segment::Tri1 => self.tri1.encode(frame, view),
            Segment::Tri2 => self.tri2.encode(frame, view),
        }
    }
}
