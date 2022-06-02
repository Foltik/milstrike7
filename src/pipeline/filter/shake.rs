use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Shake {
    t: f32,
    speed: f32,
    amount: f32,
}

pub struct ShakePass {
    filter: FilterPass,
    uniform: UniformStorage<Shake>,
}

impl ShakePass {
    pub fn new(device: &wgpu::Device, speed: f32, amount: f32) -> Self {
        let uniform = UniformStorage::new(device, "shake", Shake {
            t: 0.0,
            speed,
            amount
        });
        let filter = FilterPass::new(device, "shake", "justshake.frag.spv", Some(&uniform.uniform));

        Self {
            filter,
            uniform,
        }
    }

    pub fn speed(&mut self, speed: f32) {
        self.uniform.speed = speed;
    }

    pub fn amount(&mut self, amount: f32) {
        self.uniform.amount = amount;
    }

    pub fn update(&mut self, t: f32) {
        self.uniform.t = t;
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.filter.view(0)
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.filter.encode(frame, view);
    }
}