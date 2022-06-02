use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

pub struct AlphaPass {
    filter: FilterPass,
    uniform: UniformStorage<f32>,
}

impl AlphaPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = UniformStorage::new(device, "alpha", 1.0);
        let filter = FilterPass::new(device, "alpha", "alpha.frag.spv", Some(&uniform.uniform));

        Self {
            filter,
            uniform,
        }
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.filter.view(0)
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.filter.encode(frame, view);
    }
}

impl Deref for AlphaPass {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for AlphaPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}