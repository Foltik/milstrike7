use lib::gfx::pass::FilterPass;
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

pub struct ClearPass {
    color: wgpu::Color,
    filter: FilterPass,
}

impl ClearPass {
    pub fn new(device: &wgpu::Device, color: wgpu::Color) -> Self {
        Self {
            color,
            filter: FilterPass::new::<()>(device, "passthrough", "passthrough.frag.spv", None),
        }
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.filter.encode_with(frame, view, |a| a.color(|c| c.clear(self.color)));
    }
}
