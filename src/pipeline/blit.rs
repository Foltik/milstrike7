use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
// use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;
// use lib::math::{Matrix4, SquareMatrix};

pub struct BlitPass {
    pipeline: wgpu::RenderPipeline,
    group: wgpu::BindGroup,

    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    // uniform: UniformStorage<Matrix4>,

    dirty: Cell<bool>,
}

impl BlitPass {
    pub fn new(name: &str) -> BlitPassBuilder {
        BlitPassBuilder {
            name,

            texture: None,
            sampler: None,
            // transform: None,

            clear: None,
            color_blend: None,
            alpha_blend: None,
            write_mask: None,
        }
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        &self.view
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        // if self.dirty.get() {
        //     self.uniform.upload(frame);
        //     self.dirty.set(false);
        // }

        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |a| a)
            .begin(frame);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..3, 0..1);
    }
}

pub struct BlitPassBuilder<'a> {
    name: &'a str,

    texture: Option<wgpu::util::TextureBuilder<'a>>,
    sampler: Option<wgpu::util::SamplerBuilder<'a>>,
    // transform: Option<Matrix4>,

    clear: Option<wgpu::Color>,
    color_blend: Option<wgpu::BlendComponent>,
    alpha_blend: Option<wgpu::BlendComponent>,
    write_mask: Option<wgpu::ColorWrites>,
}

impl<'a> BlitPassBuilder<'a> {
    pub fn texture<F>(mut self, texture: F) -> Self
    where
        F: FnOnce(wgpu::util::TextureBuilder) -> wgpu::util::TextureBuilder,
    {
        let builder = wgpu::util::TextureBuilder::new("");
        self.texture = Some(texture(builder));
        self
    }

    pub fn sampler<F>(mut self, name: &'a str, sampler: F) -> Self
    where
        F: FnOnce(wgpu::util::SamplerBuilder) -> wgpu::util::SamplerBuilder,
    {
        let builder = wgpu::util::SamplerBuilder::new(name);
        self.sampler = Some(sampler(builder));
        self
    }

    // pub fn transform(mut self, matrix: Matrix4) -> Self {
    //     self.transform = Some(matrix);
    //     self
    // }

    pub fn color_blend(mut self, blend: wgpu::BlendComponent) -> Self {
        self.color_blend = Some(blend);
        self
    }

    pub fn alpha_blend(mut self, blend: wgpu::BlendComponent) -> Self {
        self.alpha_blend = Some(blend);
        self
    }

    pub fn write_mask(mut self, mask: wgpu::ColorWrites) -> Self {
        self.write_mask = Some(mask);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> BlitPass {
        let name = format!("blit_{}", self.name);

        let texture = self
            .texture
            .unwrap_or_else(|| wgpu::util::TextureBuilder::new(&name))
            .label(&name)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .build(device)
            .view()
            .build();

        let sampler = self
            .sampler
            .unwrap_or_else(|| wgpu::util::SamplerBuilder::new(&name))
            .build(device);

        // let transform = self.transform.unwrap_or_else(|| Matrix4::identity());
        // let uniform = UniformStorage::new(device, &format!("{}_transform", name), transform);

        let layout = wgpu::util::BindGroupLayoutBuilder::new(&name)
            .tex(wgpu::ShaderStages::FRAGMENT)
            .sampler(wgpu::ShaderStages::FRAGMENT)
            // .uniform(wgpu::ShaderStages::FRAGMENT)
            .build(device);

        let group = wgpu::util::BindGroupBuilder::new(&name)
            .texture(&texture)
            .sampler(&sampler)
            // .uniform(&uniform.uniform)
            .build(device, &layout);

        let vs = lib::resource::read_shader(&device, "billboard.vert.spv");
        let fs = lib::resource::read_shader(&device, "blit.frag.spv");
        let mut pipeline = wgpu::util::PipelineBuilder::new(&name)
            .with_layout(&layout)
            .render(&vs)
            .fragment(&fs);

        if let Some(blend) = self.color_blend {
            pipeline = pipeline.color_blend(blend);
        }
        if let Some(blend) = self.alpha_blend {
            pipeline = pipeline.alpha_blend(blend);
        }
        if let Some(mask) = self.write_mask {
            pipeline = pipeline.write_mask(mask);
        }

        let pipeline = pipeline.build(device);

        BlitPass {
            pipeline,
            group,

            view: texture,
            sampler,
            // uniform,

            dirty: Cell::new(false),
        }
    }
}
