use async_trait::async_trait;
use lib::gfx::animation::Animator;
use lib::prelude::*;
use lib::gfx::mesh::*;
use bytemuck::{Pod, Zeroable};

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Test {
    scene: Phong,
    animator: Animator,
    blit: BlitPass,
}

impl Test {
    pub fn new(app: &App) -> Self {
        let scene = Phong::new(app, "test.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);
        let blit = BlitPass::new("test")
            .build(&app.device);

        Self {
            scene,
            animator,
            blit,
        }
    }
}

#[async_trait]
impl Stage for Test {
    async fn init(&mut self, p: &mut Player) {
        self.animator.play(p.t(), true, "Action.001");
        self.animator.play(p.t(), true, "IcosphereAction");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.animator.update(p.t(), &mut self.scene.scene);
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {}
    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {}

    fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.encode(frame, self.blit.view());
        self.blit.encode(frame, target);
    }
}
