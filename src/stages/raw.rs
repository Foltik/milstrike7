use async_trait::async_trait;
use lib::prelude::*;
use lib::gfx::mesh::*;
use bytemuck::{Pod, Zeroable};

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Test {
    pipeline: wgpu::RenderPipeline,
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    n: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 4],
    tex: [f32; 2],
}

fn v(pos: [i8; 3], tex: [i8; 2]) -> Vertex {
    Vertex {
        pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        tex: [tex[0] as f32, tex[1] as f32]
    }
}

impl Test {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_data = [
            // top (0, 0, 1)
            v([-1, -1, 1], [0, 0]),
            v([1, -1, 1], [1, 0]),
            v([1, 1, 1], [1, 1]),
            v([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            v([-1, 1, -1], [1, 0]),
            v([1, 1, -1], [0, 0]),
            v([1, -1, -1], [0, 1]),
            v([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            v([1, -1, -1], [0, 0]),
            v([1, 1, -1], [1, 0]),
            v([1, 1, 1], [1, 1]),
            v([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            v([-1, -1, 1], [1, 0]),
            v([-1, 1, 1], [0, 0]),
            v([-1, 1, -1], [0, 1]),
            v([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            v([1, 1, -1], [1, 0]),
            v([-1, 1, -1], [0, 0]),
            v([-1, 1, 1], [0, 1]),
            v([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            v([1, -1, 1], [0, 0]),
            v([-1, -1, 1], [1, 0]),
            v([-1, -1, -1], [1, 1]),
            v([1, -1, -1], [0, 1]),
        ].to_vec();

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ].to_vec();

        let vertex = device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let vertex = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("vertices"),
        //     usage: wgpu::BufferUsages::VERTEX,
        //     size: mesh.verts.len() as wgpu::BufferAddress,
        //     mapped_at_creation: true,
        // });
        // vertex
        //     .slice(..)
        //     .get_mapped_range_mut()
        //     .copy_from_slice(&mesh.verts);
        // vertex.unmap();

        let index = device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: Some("indices"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // let index = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("indices"),
        //     usage: wgpu::BufferUsages::INDEX,
        //     size: mesh.inds.len() as wgpu::BufferAddress,
        //     mapped_at_creation: true,
        // });
        // index
        //     .slice(..)
        //     .get_mapped_range_mut()
        //     .copy_from_slice(&mesh.inds);
        // index.unmap();

        // let vs = lib::resource::read_shader(device, "billboard.vert.spv");
        // let fs = lib::resource::read_shader(device, "uv.frag.spv");
        let vs = lib::resource::read_shader(device, "mesh.vert.spv");
        let fs = lib::resource::read_shader(device, "mesh.frag.spv");

        let pipeline = wgpu::util::PipelineBuilder::new("test")
            .render(&vs)
            .fragment(&fs)
            .add_vertex_buffer_layout(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 4 * 4,
                        shader_location: 1,
                    },
                ],
            })
            // .add_vertex_buffer::<Vertex>(Vertex::ty().attrs())
            .build(device);

        Self {
            pipeline,
            vertex,
            index,
            n: index_data.len() as u32,
        }
    }
}

#[async_trait]
impl Stage for Test {
    async fn init(&mut self, p: &mut Player) {}

    async fn update(&mut self, p: &mut Player, dt: f32) {
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(view, |c| c.color(|col| col.clear(wgpu::Color::BLUE)))
            .begin(frame);

        pass.set_pipeline(&self.pipeline);

        pass.set_vertex_buffer(0, self.vertex.slice(..));
        pass.set_index_buffer(self.index.slice(..), wgpu::IndexFormat::Uint16);

        // pass.draw(0..3, 0..1);
        pass.draw_indexed(0..self.n, 0, 0..1);
    }
}
