use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Thanks {
    image: ImagePass,
}

impl Thanks {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let image = ImagePass::new(device)
            .with(app, "credits.png", "credits", v2(0.0, 0.0), v2(1.0, 1.0));

        Self {
            image,
        }
    }
}

#[async_trait]
impl Stage for Thanks {
    async fn init(&mut self, p: &mut Player) {}

    async fn update(&mut self, p: &mut Player, dt: f32) {}

    async fn event(&mut self, p: &mut Player, ev: Event) {}

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {}

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.image.encode(frame, view);
    }
}
