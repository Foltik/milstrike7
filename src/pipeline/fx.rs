use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

use lib::midi2::device::worlde_easycontrol9::Input;

use crate::Time;
use crate::{AlphaPass, BloomPass};

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Fx {
    pub t: f32,
    pub tc: f32,
    pub pause: f32,
    pub glitch: f32,
    pub glitch_mo: f32,
    pub vhs: f32,
    pub red: f32,
    pub flash: f32,
    pub bloom: f32,
    pub invert: f32,
    pub edge: f32,
    pub mega: f32,
}

pub struct FxPass {
    pub state: UniformStorage<Fx>,

    edge: FilterPass,
    shake: FilterPass,
    glitch: FilterPass,
    vhs: FilterPass,
    pause: FilterPass,

    invert: FilterPass,
    invert_u: UniformStorage<f32>,


    bloom: BloomPass,
    pub alpha: AlphaPass,
}

impl FxPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize)) -> Self {
        let state = UniformStorage::new(device, "fx", Fx::default());

        let edge = FilterPass::new_sized(device, "edge", "edge.frag.spv", Some(state.as_ref()), size);
        let shake = FilterPass::new_sized(device, "shake", "shake.frag.spv", Some(state.as_ref()), size);
        let glitch = FilterPass::new_sized(device, "glitch", "glitch.frag.spv", Some(state.as_ref()), size);
        let vhs = FilterPass::new_sized(device, "vhs", "vhs.frag.spv", Some(state.as_ref()), size);
        let pause = FilterPass::new_sized(device, "pause", "pause.frag.spv", Some(state.as_ref()), size);

        let invert_u = UniformStorage::new(device, "invert", 0.0);
        let invert = FilterPass::new_sized(device, "invert", "invert.frag.spv", Some(&invert_u.uniform), size);

        Self {
            state,
            edge,
            shake,
            glitch,
            vhs,
            pause,

            invert,
            invert_u,

            bloom: BloomPass::new(device, size, 0.0),
            alpha: AlphaPass::new(device),
        }
    }

    pub fn update(&mut self, time: &Time) {
        self.state.t = time.t_rms();
        self.state.tc = time.t();
        *self.invert_u = self.state.invert;
        *self.bloom = self.state.bloom;
    }

    pub fn ctrl(&mut self, input: Input) {
        match input {
            Input::Slider(0, f) => *self.alpha = f,
            Input::Slider(1, f) => self.state.bloom = f,
            Input::Slider(2, f) => self.state.edge = f,
            Input::Slider(3, f) => self.state.glitch = f,
            Input::Slider(4, f) => self.state.vhs = f,
            // Input::Slider(5, f) => self.state.pause = f,
            _ => {},
        }
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.edge.view(0)
    }

    pub fn upload(&self, frame: &mut Frame) {
        self.state.upload(frame);
        self.invert_u.upload(frame);
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.edge.encode(frame, self.bloom.view());
        self.bloom.encode(frame, self.shake.view(0));
        self.shake.encode(frame, self.glitch.view(0));
        self.glitch.encode(frame, self.vhs.view(0));
        self.vhs.encode(frame, self.pause.view(0));
        self.pause.encode(frame, self.invert.view(0));
        self.invert.encode(frame, self.alpha.view());
        self.alpha.encode(frame, view);
    }
}

impl std::ops::Deref for FxPass {
    type Target = Fx;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for FxPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
