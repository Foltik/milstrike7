use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FlyTorus {
    pub color: [f32; 3],
    pub t: f32,
    pub speed: f32,
    pub mx: f32,
    pub my: f32,
    pub warp: f32,
}

pub struct FlyTorusPass {
    synth: SynthPass,
    uniform: UniformStorage<FlyTorus>,
}

impl FlyTorusPass {
    pub fn new(device: &wgpu::Device, wormhole: FlyTorus) -> Self {
        let uniform = UniformStorage::new(device, "wormhole", wormhole);
        let synth = SynthPass::new(device, "wormhole", "wormhole.frag.spv", Some(&uniform.uniform));
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

impl Deref for FlyTorusPass {
    type Target = FlyTorus;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for FlyTorusPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}