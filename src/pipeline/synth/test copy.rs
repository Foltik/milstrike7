use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Test {
    pub t: f32,
}

pub struct TestPass {
    synth: SynthPass,
    uniform: UniformStorage<Test>,
}

impl TestPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = UniformStorage::new(device, "test", Test {
            t: 0.0,
        });
        let synth = SynthPass::new(device, "test", "scanlines.frag.spv", Some(&uniform.uniform));
        Self {
            synth,
            uniform,
        }
    }

    pub fn update(&mut self, t: f32) {
        self.uniform.t = t;
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, view);
    }
}

impl Deref for TestPass {
    type Target = Test;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for TestPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}