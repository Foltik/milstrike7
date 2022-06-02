use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

pub struct ScanlinesPass {
    synth: SynthPass,
    uniform: UniformStorage<f32>,
}

impl ScanlinesPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = UniformStorage::new(device, "scanlines", 0.0);
        let synth = SynthPass::new(device, "scanlines", "scanlines.frag.spv", Some(&uniform.uniform));
        Self {
            synth,
            uniform,
        }
    }

    pub fn update(&mut self, t: f32) {
        *self.uniform = t;
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, view);
    }
}