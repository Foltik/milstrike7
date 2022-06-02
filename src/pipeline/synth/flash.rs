use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::time::Decay;

use super::ColorPass;

pub struct FlashPass {
    base: [f32; 3],
    color: ColorPass,
    decay: Decay,
}

impl FlashPass {
    pub fn new(device: &wgpu::Device, ms: f32, base: [f32; 3]) -> Self {
        Self {
            base,
            color: ColorPass::new(device, base),
            decay: Decay::new(ms),
        }
    }

    pub fn set(&mut self) {
        self.decay.set();
    }

    pub fn set_fr(&mut self, fr: f32) {
        self.decay.set_fr(fr);
    }

    pub fn hold(&mut self) {
        self.decay.set();
        self.decay.hold(true);
    }

    pub fn release(&mut self) {
        self.decay.hold(false);
    }

    pub fn update(&mut self, t: f32) {
        self.decay.update(t);

        let fr = self.decay.v();
        let (r, g, b) = (fr * self.base[0], fr * self.base[1], fr * self.base[2]);
        self.color.color(&[r, g, b]);
    }

    pub fn color(&mut self, color: [f32; 3]) {
        self.base = color;
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.color.encode(frame, view);
    }
}