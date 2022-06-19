use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Sanctuary {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator: Animator,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,

}

impl Sanctuary {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default();
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("template.cfg");

        let scene = Phong::new(app, "test.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "template_composite",
            2,
            Some("composite_add.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("template").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            scene,
            animator,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Sanctuary {
    async fn init(&mut self, p: &mut Player) {
        // self.animator.play(self.t, true, "");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 50.0 * p.rms() * self.t_mul * dt;

        self.animator.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Trigger { id: 10 } => p.go("funky_beat").await,
            _ => {}
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            _ => {}
        }
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        let decay = &self.decay;
        let count = &self.count;
        let cfg = &self.cfg;

        self.scene.encode(frame, self.composite.view(0));

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
