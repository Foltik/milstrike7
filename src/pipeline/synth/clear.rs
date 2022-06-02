use lib::gfx::pass::SynthPass;
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

pub struct ClearPass {
    synth: SynthPass,
}

impl ClearPass {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            synth: SynthPass::new::<()>(device, "clear", "black.frag.spv", None),
        }
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.synth.encode(frame, view);
    }
}