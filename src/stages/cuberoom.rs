use async_trait::async_trait;
use lib::gfx::animation::Animator;
use lib::prelude::*;
use lib::gfx::mesh::*;
use bytemuck::{Pod, Zeroable};

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct CubeRoom {
    scene: Phong,
    animator: Animator,
    blit: BlitPass,

    t: f32,
    t_mul: f32,
}

impl CubeRoom {
    pub fn new(app: &App) -> Self {
        let scene = Phong::new(app, "cuberoom.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);
        let blit = BlitPass::new("cuberoom")
            .build(&app.device);

        Self {
            scene,
            animator,
            blit,

            t: 0.0,
            t_mul: 1.0,
        }
    }
}

#[async_trait]
impl Stage for CubeRoom {
    async fn init(&mut self, p: &mut Player) {
        self.animator.play(self.t, true, "CubeAction");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 50.0 * p.rms() * self.t_mul * dt;

        self.animator.update(self.t, &mut self.scene.scene);


    }

    async fn event(&mut self, p: &mut Player, ev: Event) {}
    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {}

    fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.encode(frame, self.blit.view());
        self.blit.encode(frame, target);
    }
}
