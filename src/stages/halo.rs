use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Halo {
    segment: Segment,

    t: f32,
    t_mul: f32,
    angle: Euler<Rad<f32>>,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator1: Animator,
    animator2: Animator,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,

}

impl Halo {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("bigkick", 0.0)
            .with("bigsnare", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("halo.cfg");

        let scene = Phong::new(app, "halo.glb", |_node| true, |_mat| true);
        let animator1 = Animator::new(&scene.scene);
        let animator2 = Animator::new(&scene.scene);

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "halo_composite",
            2,
            Some("composite_add.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("halo").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 0.0,
            angle: Euler {
                x: Rad(0.0),
                y: Rad(0.0),
                z: Rad(0.0),
            },

            decay,
            count,
            cfg,

            scene,
            animator1,
            animator2,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Halo {
    async fn init(&mut self, p: &mut Player) {
        self.animator1.play(p.t(), false, "Camera Intro Pan");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.animator1.update(p.t(), &mut self.scene.scene);
        self.animator2.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);

        let mul = self.cfg.f32("mul");
        self.angle.x += Rad(mul * p.rms() * self.t_mul * dt * self.cfg.f32("x"));
        self.angle.y += Rad(mul * p.rms() * self.t_mul * dt * self.cfg.f32("y"));
        self.angle.z += Rad(mul * p.rms() * self.t_mul * dt * self.cfg.f32("z"));
        self.scene.node("Halo").transform.rotate = self.angle.into();

        self.fx.shake = self.decay.v("bigkick") * self.cfg.f32("shake");
        self.fx.flash = self.decay.v("bigsnare") * self.cfg.f32("flash");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 60, t } => decay.set_t("bigkick", t * 0.5),
            Event::Beat { id: 61, t } => decay.set_t("bigsnare", t),

            // Event::Trigger { id: 29 } => self.animator1.play(p.t(), false, "Camera Intro Pan"),
            Event::Trigger { id: 28 } => {
                self.t_mul = 1.0;
                self.animator1.play(p.t(), false, "Camera Intro Impact")
            },
            Event::Trigger { id: 10 } => p.go("aqua").await,

            Event::Mod { id: 0, fr } => {
                self.fx.vhs = fr;
                self.fx.edge = fr;
                self.fx.glitch = fr;
            }

            _ => {}
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            Key::Key1 => self.animator1.play(p.t(), false, "Camera Intro Pan"),
            Key::Key2 => {
                self.animator1.play(p.t(), false, "Camera Intro Impact");
                self.animator1.play(p.t(), false, "Skybox Intro Impact");
            },
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
