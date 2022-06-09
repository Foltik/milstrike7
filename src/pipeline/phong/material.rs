use lib::app::App;

use lib::math::{Vector2, Vector4};

use lib::gfx::material::{MaterialAttr, MaterialDesc};
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialUniform {
    pub color: Vector4,
    pub scale: Vector2,
    pub mapped: u32,
    pub unlit: u32,
}

pub struct Material {
    pub name: String,
    pub uniform: UniformStorage<MaterialUniform>,
    group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        app: &App,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        desc: &MaterialDesc,
    ) -> Self {
        let (image, scale) = match &desc.color {
            MaterialAttr::Value(_) => {
                wgpu::util::image::load(app, &lib::resource::read_image("missing.png"))
            }
            MaterialAttr::Map(img) => wgpu::util::image::load(app, img),
        };

        let uniform = UniformStorage::new(
            &app.device,
            &format!("material_{}", &desc.name),
            MaterialUniform {
                color: match &desc.color {
                    MaterialAttr::Value(v) => *v,
                    _ => Vector4::new(1.0, 1.0, 1.0, 1.0),
                },
                scale,
                mapped: if let MaterialAttr::Map(_) = &desc.color {
                    1
                } else {
                    0
                },
                unlit: if desc.unlit { 1 } else { 0 },
            },
        );

        let group = wgpu::util::BindGroupBuilder::new(&desc.name)
            .texture(&image.view().build())
            .sampler(sampler)
            .uniform(&uniform.as_ref())
            .build(&app.device, layout);

        Self {
            name: desc.name.clone(),
            uniform,
            group,
        }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);
    }
}
