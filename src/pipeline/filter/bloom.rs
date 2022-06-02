use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

pub struct BloomPass {
    passthrough: FilterPass,

    bloom: FilterPass,
    bloom_u: UniformStorage<(f32, f32)>,

    tile: FilterPass,
    tile_u: UniformStorage<f32>,
}

impl BloomPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), amount: f32) -> Self {
        let passthrough = FilterPass::new_sized::<()>(device, "passthrough", "passthrough.frag.spv", None, size);

        let bloom_u = UniformStorage::new(device, "bloom", (size.0 as f32, size.1 as f32));
        let bloom = FilterPass::new_sized(device, "bloom", "bloom.frag.spv", Some(&bloom_u.uniform), size);

        let tile_u = UniformStorage::new(device, "bloom_tile", amount);
        let tile = FilterPass::new_composite_sized(device, "bloom_tile", 2, Some("bloom_tile.frag.spv"), Some(&tile_u.uniform), size);

        Self {
            passthrough,

            bloom,
            bloom_u,

            tile,
            tile_u,
        }
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.passthrough.view(0)
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.tile_u.upload(frame);

        self.passthrough.encode(frame, self.bloom.view(0));
        self.passthrough.encode(frame, self.tile.view(0));

        self.bloom.encode(frame, self.tile.view(1));
        self.tile.encode(frame, view);
    }
}

impl Deref for BloomPass {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        self.tile_u.deref()
    }
}

impl DerefMut for BloomPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tile_u.deref_mut()
    }
}