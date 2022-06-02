use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use rand::Rng;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Digits {
    pub color: [f32; 3],
    pub t: f32,
    pub i: u32,
}

pub struct DigitsPass {
    synth: SynthPass,
    uniform: UniformStorage<Digits>,
}

impl DigitsPass {
    pub fn new(device: &wgpu::Device, color: [f32; 3]) -> Self {
        let uniform = UniformStorage::new(device, "digits", Digits {
            color,
            t: 0.0,
            i: (rand::thread_rng().gen::<f32>() * 1000.0).floor() as u32,
        });
        let synth = SynthPass::new(device, "digits", "digits.frag.spv", Some(&uniform.uniform));
        Self {
            synth,
            uniform,
        }
    }

    pub fn permute(&mut self) {
        self.uniform.i += 1;
    }

    pub fn update(&mut self, t: f32) {
        self.uniform.t = t;
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, view);
    }
}

impl Deref for DigitsPass {
    type Target = Digits;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for DigitsPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}