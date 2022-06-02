use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Substrate {
    pub color: [f32; 3],
    pub t: f32,
    pub mx: f32,
    pub my: f32,
    pub amt: f32,
}

pub struct SubstratePass {
    synth: SynthPass,
    uniform: UniformStorage<Substrate>,
}

impl SubstratePass {
    pub fn new(device: &wgpu::Device, substrate: Substrate) -> Self {
        let uniform = UniformStorage::new(device, "substrate", substrate);
        let synth = SynthPass::new(device, "substrate", "substrate.frag.spv", Some(&uniform.uniform));
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

impl Deref for SubstratePass {
    type Target = Substrate;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for SubstratePass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}