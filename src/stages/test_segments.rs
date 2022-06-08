use lib::prelude::*;
use async_trait::async_trait;

use crate::pipeline::{IsoTriPass, IsoTri};
use crate::demo::{Event, Player, Stage};

pub struct TestSegments {
    strobe: Decay,
    beat: Decay,
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
            strobe: Decay::new(25.0),
            beat: Decay::new(200.0),
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
        self.beat.update(dt);
        self.strobe.update(dt);

        if self.beat.off() {
            tri.r = 0.0;
            tri.thickness = 0.5 + 0.5 * self.strobe.v();
            tri.weight = 0.5;
        } else {
            tri.r = self.beat.v();
            tri.thickness = 0.5 + self.beat.v();
            tri.weight = 0.5 + 0.5 * self.beat.v();
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        match ev {
            Event::Snare => self.strobe.set(),
            Event::Kick => self.beat.set(),
            Event::Strobe => self.t_mul = 3.0,
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
