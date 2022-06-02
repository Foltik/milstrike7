use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

use crate::pipeline::{ScanlinesPass, BloomPass};

pub struct TestPass {
    synth: ScanlinesPass,
    bloom: BloomPass,
}

impl TestPass {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            synth: ScanlinesPass::new(device),
            bloom: BloomPass::new(device, (1920, 1080), 1.0),
        }
    }

    pub fn update(&mut self, t: f32) {
        self.synth.update(t);
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.synth.encode(frame, self.bloom.view());
        self.bloom.encode(frame, view);
    }
}
