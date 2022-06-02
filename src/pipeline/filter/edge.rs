use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

pub struct EdgePass {
    filter: FilterPass,
    uniform: UniformStorage<(f32, f32, f32)>,
}

impl EdgePass {
    pub fn new(device: &wgpu::Device, size: (usize,  usize), fr: f32) -> Self {
        let uniform = UniformStorage::new(device, "justedge", (size.0 as f32, size.1 as f32, fr));
        let filter = FilterPass::new(device, "justedge", "justedge.frag.spv", Some(&uniform.uniform));

        Self {
            filter,
            uniform,
        }
    }
    
    pub fn fr(&mut self, fr: f32) {
        self.uniform.2 = fr;
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.filter.view(0)
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.filter.encode(frame, view);
    }
}