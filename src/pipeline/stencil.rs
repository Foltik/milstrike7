use std::collections::HashMap;

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::uniform::{Uniform, UniformArray};
use lib::gfx::pass::{SynthPass, FilterPass};
use lib::app::App;

use super::ClearPass;

const BILLBOARD_SHADER: &str = "billboard.vert.spv";
const COMPOSITE_SHADER: &str = "composite_add.frag.spv";
const CLEAR_SHADER: &str = "black.frag.spv";
const STENCIL_SHADER: &str = "abyss_stencil.frag.spv";

type View = wgpu::RawTextureView;

struct Stencil {
    width: usize,
    height: usize,
    view: View,
}

impl Stencil {
    fn view(&self) -> &View {
        &self.view
    }
}

pub struct StencilPass {
    stencils: HashMap<String, Stencil>,

    image_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    base: FilterPass,
    composite: FilterPass,
    clear: ClearPass,
}

impl StencilPass {
    pub const N: usize = 29;

    pub fn new(
        device: &wgpu::Device,
    ) -> Self {
        let vs = lib::resource::read_shader(device, BILLBOARD_SHADER);
        let fs = lib::resource::read_shader(device, STENCIL_SHADER);

        let mut names = Vec::new();
        let mut views = Vec::new();
        let mut sizes = Vec::new();

        let mut transforms = Vec::new();

        let mut stencil = |name: &str, pos: (usize, usize), size: (usize, usize)| {
            let view = wgpu::util::TextureBuilder::new(&format!("stencil_input_{}", name))
                .size([size.0 as u32, size.1 as u32, 1u32])
                .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
                .build(device)
                .view()
                .build()
                .into_raw();

            names.push(name.to_owned());
            views.push(view);
            sizes.push(size);

            transforms.push(Transform {
                x: pos.0 as f32 / 1920.0,
                y: pos.1 as f32 / 1080.0,
                w: size.0 as f32 / 1920.0,
                h: size.1 as f32 / 1080.0,
            });
        };

        stencil("lights",    (602, 30),   (715, 15));

        stencil("center",    (625, 345),  (670, 390));

        stencil("dj",        (800,  824), (320, 162));

        stencil("djr_top",   (960,  760), (320, 64));
        stencil("djr_mid",   (1120, 865), (320, 80));
        stencil("djr_bot",   (960,  986), (320, 64));
        stencil("djr_pod",   (1280, 986), (520, 80));

        stencil("djl_top",   (640, 760),  (320, 64));
        stencil("djl_mid",   (480, 865),  (320, 80));
        stencil("djl_bot",   (640, 986),  (320, 64));
        stencil("djl_pod",   (120, 986),  (520, 80));

        stencil("sr_pipe1",  (1680, 0),   (240, 180));
        stencil("sr_pipe2",  (1680, 212), (240, 180));
        stencil("sr_pipe3",  (1680, 420), (240, 180));
        stencil("sr_pipe4",  (1680, 630), (240, 180));
        stencil("sr_inner1", (1569, 150), (111, 180));
        stencil("sr_inner2", (1569, 420), (111, 180));
        stencil("sr_inner3", (1569, 685), (111, 180));
        stencil("sr_top",    (1330, 0),   (220, 212));
        stencil("sr_wing",   (1315, 225), (250, 630));

        stencil("sl_pipe1",  (0, 0),      (240, 180));
        stencil("sl_pipe2",  (0, 212),    (240, 180));
        stencil("sl_pipe3",  (0, 420),    (240, 180));
        stencil("sl_pipe4",  (0, 630),    (240, 180));
        stencil("sl_inner1", (240, 150),  (111, 180));
        stencil("sl_inner2", (240, 420),  (111, 180));
        stencil("sl_inner3", (240, 685),  (111, 180));
        stencil("sl_top",    (370, 0),    (220, 212));
        stencil("sl_wing",   (355, 225),  (250, 630));

        let n = views.len();

        let count = Uniform::new(device, "stencil_count", Some(&n));
        let transforms = UniformArray::new(device, "stencil_transforms", n, Some(&transforms));

        let sampler = wgpu::util::SamplerBuilder::new("stencil_sampler")
            .build(device);

        let image_layout =
            wgpu::util::BindGroupLayoutBuilder::new("stencil_image")
                .textures(wgpu::ShaderStages::FRAGMENT, n as usize)
                .sampler(wgpu::ShaderStages::FRAGMENT)
                .uniform(wgpu::ShaderStages::FRAGMENT)
                .uniform(wgpu::ShaderStages::FRAGMENT)
                .build(device);

        let view_refs = views.iter().collect::<Vec<_>>();
        let image_group = wgpu::util::BindGroupBuilder::new("stencil_image")
            .textures(&view_refs)
            .sampler(&sampler)
            .uniform(&count)
            .uniform_array(&transforms)
            .build(device, &image_layout);

        let pipeline = wgpu::util::PipelineBuilder::new("stencil")
            .with_layout(&image_layout)
            .render(&vs)
            .fragment(&fs)
            .build(device);
        
        let mut stencils = HashMap::new();
        for (name, view, (width, height)) in itertools::multizip((names, views, sizes)) {
            stencils.insert(name, Stencil {
                width,
                height,
                view,
            });
        }

        Self {
            stencils,

            image_group,
            pipeline,

            base: FilterPass::new_passthrough(device),
            composite: FilterPass::new_composite::<()>(device, "stencil_composite", 2, Some(COMPOSITE_SHADER), None),
            clear: ClearPass::new(device, wgpu::Color::BLACK),
        }
    }

    pub fn view(&self, name: &str) -> &View {
        self.stencils[name].view()
    }

    pub fn views<'a, I>(&self, names: I) -> impl Iterator<Item = &View>
    where
        I: IntoIterator<Item = &'a str>
    {
        names.into_iter()
            .map(str::to_owned)
            .map(move |n| self.view(&n))
    }

    pub fn all_views(&self) -> impl Iterator<Item = &View> {
        self.stencils.values().map(|s| s.view())
    }

    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a String> {
        self.stencils.keys()
    }

    pub fn base(&self) -> &View { self.base.view(0) }

    pub fn lights(&self) -> &View { self.view("lights") }

    pub fn center(&self) -> &View { self.view("center") }

    pub fn dj(&self) -> &View { self.view("dj") }

    pub fn djr_top(&self) -> &View { self.view("djr_top") }
    pub fn djr_mid(&self) -> &View { self.view("djr_mid") }
    pub fn djr_bot(&self) -> &View { self.view("djr_bot") }
    pub fn djr_pod(&self) -> &View { self.view("djr_pod") }

    pub fn djl_top(&self) -> &View { self.view("djl_mid") }
    pub fn djl_mid(&self) -> &View { self.view("djl_mid") }
    pub fn djl_bot(&self) -> &View { self.view("djl_bot") }
    pub fn djl_pod(&self) -> &View { self.view("djl_pod") }

    pub fn dj_all(&self) -> impl Iterator<Item = &View> { self.views(["dj", "djr_top", "djr_mid", "djr_bot", "djr_pod", "djl_top", "djl_mid", "djl_bot", "djl_pod"]) }

    pub fn sl_top(&self) -> &View { self.view("sr_top") }
    pub fn sl_pipes(&self) -> impl Iterator<Item = &View> { self.views(["sl_pipe1", "sl_pipe2", "sl_pipe3", "sl_pipe4"]) }
    pub fn sl_inner(&self) -> impl Iterator<Item = &View> { self.views(["sl_inner1", "sl_inner2", "sl_inner3"]) }

    pub fn sr_top(&self) -> &View { self.view("sr_top") }
    pub fn sr_pipes(&self) -> impl Iterator<Item = &View> { self.views(["sr_pipe1", "sr_pipe2", "sr_pipe3", "sr_pipe4"]) }
    pub fn sr_inner(&self) -> impl Iterator<Item = &View> { self.views(["sr_inner1", "sr_inner2", "sr_inner3"]) }

    pub fn sl_wing(&self) -> &View { self.view("sl_wing") }
    pub fn sr_wing(&self) -> &View { self.view("sr_wing") }

    pub fn pipes(&self) -> impl Iterator<Item = &View> { self.sl_pipes().chain(self.sr_pipes()) }
    pub fn top(&self) -> impl Iterator<Item = &View> { self.views(["sl_top", "sr_top"]) }
    pub fn inner(&self) -> impl Iterator<Item = &View> { self.sl_inner().chain(self.sr_inner()) }

    pub fn size(&self, name: &str) -> (usize, usize) {
        let s = &self.stencils[name];
        (s.width, s.height)
    }

    pub async fn clear(&self, app: &App) {
        app.frame(|frame| {
            self.clear.encode(frame, self.base());
            for v in self.all_views() {
                self.clear.encode(frame, v);
            }
        }).await;
    }

    pub fn encode(&self, frame: &mut Frame, view: &View) {
        {
            let mut pass = wgpu::util::RenderPassBuilder::new()
                .color_attachment(self.composite.view(0), |b| b)
                .begin(frame);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.image_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.base.encode(frame, self.composite.view(1));

        self.composite.encode(frame, view);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Transform {
    pub x: f32,
    pub y: f32,

    pub w: f32,
    pub h: f32,
}
