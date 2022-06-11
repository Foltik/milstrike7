use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::uniform::{UniformArrayStorage, Uniform};

pub struct TypographyPass {
    uniform_text: UniformArrayStorage<TextUniform>,
    uniform_count: Uniform<u32>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TextUniform {
}

impl Default for TextUniform {
    fn default() -> Self {
        Self {
        }
    }
}

pub struct Text {
    text: String,
}

impl TypographyPass {
    pub const MAX: usize = 16;

    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_text = UniformArrayStorage::new(device, "typography_text", Self::MAX, None);
        let uniform_count = Uniform::new(device, "typography_count", Some(&0));

        Self {
            uniform_text,
            uniform_count,
        }
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform_text.upload(frame);
        self.uniform_count.upload(frame, &0); // TODO
    }
}
