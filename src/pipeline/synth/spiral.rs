use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Spiral {
    pub color: [f32; 3],
    pub aspect: f32,
    pub t: f32,
    pub swirl: f32,
    pub speed: f32,
    pub cutoff: f32,
    pub amount: f32,
    pub spokes: u32,
}

pub struct SpiralPass {
    synth: SynthPass,
    uniform: UniformStorage<Spiral>,
}

impl SpiralPass {
    pub fn new(device: &wgpu::Device, spiral: Spiral) -> Self {
        let uniform = UniformStorage::new(device, "spiral", spiral);
        let synth = SynthPass::new(device, "spiral", "spiral.frag.spv", Some(&uniform.uniform));
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

impl Deref for SpiralPass {
    type Target = Spiral;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for SpiralPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}
