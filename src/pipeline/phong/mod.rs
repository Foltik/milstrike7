use lib::app::App;

use lib::gfx::frame::Frame;
use lib::gfx::mesh::{Vertex, VertexExt as _};
use lib::gfx::scene::Scene;
use lib::gfx::wgpu;

mod material;
use material::Material;

pub struct Phong {
    pipeline: wgpu::RenderPipeline,
    depth: wgpu::TextureView,
    pub mat_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,

    pub scene: Scene,
    pub mats: Vec<Material>,
    pub meshes: Vec<Vec<usize>>,
}

impl Phong {
    pub fn new<NodeFn, MatFn>(app: &App, scene: &str, mut node_filter: NodeFn, mut mat_filter: MatFn) -> Self
    where
        NodeFn: FnMut(&str) -> bool,
        MatFn: FnMut(&str) -> bool,
    {
        let scene = lib::resource::read_scene(&app.device, scene);

        let vs = lib::resource::read_shader(&app.device, "phong.vert.spv");
        let fs = lib::resource::read_shader(&app.device, "phong.frag.spv");

        let mat_layout = wgpu::util::BindGroupLayoutBuilder::new("phong")
            .tex(wgpu::ShaderStages::FRAGMENT)
            .sampler(wgpu::ShaderStages::FRAGMENT)
            .uniform(wgpu::ShaderStages::FRAGMENT)
            .build(&app.device);

        let depth = wgpu::util::TextureBuilder::new_depth("phong_depth")
            .build(&app.device)
            .view()
            .build();

        let sampler = wgpu::util::SamplerBuilder::new("phong")
            .address_mode(wgpu::AddressMode::Repeat)
            .mag_filter(wgpu::FilterMode::Nearest)
            .build(&app.device);

        let pipeline = wgpu::util::PipelineBuilder::new("phong")
            .with_layout(&scene.cam_layout)
            .with_layout(&scene.light_layout)
            .with_layout(&mat_layout)
            .with_layout(&scene.mesh_layout)
            .render(&vs)
            .fragment(&fs)
            .add_vertex_buffer::<Vertex>(Vertex::ty().attrs())
            .hack_add_default_depth_stencil_state()
            // .depth_stencil()
            .build(&app.device);

        let mats = scene
            .desc
            .materials
            .iter()
            .filter(|mat| mat_filter(&mat.name))
            .map(|mat| Material::new(app, &mat_layout, &sampler, mat))
            .collect();

        let meshes = scene
            .desc
            .materials
            .iter()
            .enumerate()
            .map(|(i, _)| {
                scene
                    .desc
                    .meshes
                    .iter()
                    .enumerate()
                    .filter(|(j, m)| {
                        m.material == i && node_filter(&scene.desc.nodes[scene.mesh_idxs[*j]].name)
                    })
                    .map(|(j, _)| j)
                    .collect()
            })
            .collect();

        Self {
            pipeline,
            depth,
            mat_layout,
            sampler,

            scene,
            mats,
            meshes,
        }
    }

    pub fn encode(
        &self,
        frame: &mut Frame,
        target: &wgpu::RawTextureView,
    ) {
        self.scene.update(frame);

        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |c| c)
            .depth_stencil_attachment(&self.depth, |d| d)
            .begin(frame);

        self.encode_to_pass(&mut pass, target);
    }

    pub fn encode_load(
        &self,
        frame: &mut Frame,
        target: &wgpu::RawTextureView,
    ) {
        self.scene.update(frame);

        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |c| c.color(|op| op.load()))
            .depth_stencil_attachment(&self.depth, |d| d)
            .begin(frame);

        self.encode_to_pass(&mut pass, target);
    }

    fn encode_to_pass<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        target: &wgpu::RawTextureView,
    ) {
        pass.set_pipeline(&self.pipeline);

        self.scene.cam.bind(pass, 0);

        self.scene.lights.bind(pass, 1);

        for (mat, meshes) in self.mats.iter().zip(self.meshes.iter()) {
            mat.bind(pass, 2);

            for i in meshes {
                self.scene.meshes[*i].draw(pass, 3);
            }
        }
    }
}
