use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Tunnel {
    color: [f32; 3],
    t: f32,
    w: f32,
    h: f32,

}
pub struct TunnelPass {
    synth: SynthPass,
    uniform: UniformStorage<Tunnel>,
}

impl TunnelPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), color: [f32; 3]) -> Self {
        let uniform = UniformStorage::new(device, "tunnel", Tunnel {
            color,
            t: 0.0,
            w: size.0 as f32,
            h: size.1 as f32,
        });
        let synth = SynthPass::new(device, "tunnel", "tunnel.frag.spv", Some(&uniform.uniform));

        Self {
            synth,
            uniform
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