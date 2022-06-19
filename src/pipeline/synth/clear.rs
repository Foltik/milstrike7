use lib::gfx::pass::SynthPass;
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

pub struct ClearPass {
    color: wgpu::Color,
    synth: SynthPass,
}

impl ClearPass {
    pub fn new(device: &wgpu::Device, color: wgpu::Color) -> Self {
        Self {
            color,
            synth: SynthPass::new::<()>(device, "clear", "clear.frag.spv", None)
        }
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.synth.encode(frame, view);
    }
}
