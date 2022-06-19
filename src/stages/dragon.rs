use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Dragon {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator: Animator,
    tri: IsoTriPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
}

impl Dragon {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("kick", 0.0)
            .with("snare", 0.0)
            .with("hat", 0.0)
            ;

        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("dragonage.cfg");

        let scene = Phong::new(app, "dragonage.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);

        let tri = IsoTriPass::new(device, IsoTri {
            color: [1.0, 0.0, 0.0],
            aspect: 16.0 / 9.0,
            t: 0.0,
            r: -3.0,
            weight: 0.2,
            thickness: 0.0,
        });

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "dragon_composite",
            2,
            Some("composite.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("dragon").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            scene,
            animator,
            tri,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Dragon {
    async fn init(&mut self, p: &mut Player) {
        *self.fx.alpha = 0.0;
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.animator.update(p.t(), &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);

        self.tri.update(self.t);
        // self.tri.thickness = match self.segment {
        //     Segment::Init => self.decay.v("beat") * self.cfg.f32("weight0"),
        //     Segment::Bass => {
        //         let min = self.cfg.f32("weightmin");
        //         let max = self.cfg.f32("weightmax");
        //         min + self.decay.v("weight") * (max - min)
        //     },
        // }

        self.scene.node("Sword0").transform.rotate = Euler {
            x: Rad(PI),
            y: Rad(self.t * self.cfg.f32("rot0")),
            z: Rad(0.0)
        }.into();

        self.scene.node("Sword1").transform.rotate = Euler {
            x: Rad(PI),
            y: Rad(self.t * self.cfg.f32("rot1")),
            z: Rad(0.0)
        }.into();

        self.scene.node("Sword2").transform.rotate = Euler {
            x: Rad(PI),
            y: Rad(self.t * self.cfg.f32("rot2")),
            z: Rad(0.0)
        }.into();

        self.fx.edge = self.decay.v("kick") * self.cfg.f32("edge");
        self.fx.shake = self.decay.v("kick") * self.cfg.f32("shake");
        self.fx.flash = self.decay.v("hat") * self.cfg.f32("flash");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 61, t } => self.decay.set_t("kick", t * 2.0),
            Event::Beat { id: 62, t } => self.decay.set_t("snare", t * 2.0),
            Event::Beat { id: 63, t } => self.decay.set_t("snare", t * 2.0),
            Event::Beat { id: 64, t } => self.decay.set_t("hat", t),
            Event::Beat { id: 65, t } => self.decay.set_t("hat", t),

            Event::Trigger { id: 23 } => self.scene.node("Camera0").transform = self.scene.node("Camera4").transform,
            Event::Trigger { id: 22 } => self.scene.node("Camera0").transform = self.scene.node("Camera3").transform,
            Event::Trigger { id: 21 } => self.scene.node("Camera0").transform = self.scene.node("Camera2").transform,
            Event::Trigger { id: 20 } => self.scene.node("Camera0").transform = self.scene.node("Camera1").transform,
            Event::Trigger { id: 19 } => self.scene.node("Camera0").transform = self.scene.node("Camera5").transform,

            Event::Trigger { id: 24 } => {
                self.animator.play(p.t(), false, "Sword Rise Into Frame");
                self.animator.play(p.t(), false, "Sword Rise Into Frame.001");
            },
            Event::Trigger { id: 18 } => {
                self.animator.play(p.t(), false, "Sword Up For Camera");
                self.animator.play(p.t(), false, "Sword Up For Camera.001");
            },
            Event::Trigger { id: 17 } => {
                self.animator.play(p.t(), false, "Sword Down for Camera");
                self.animator.play(p.t(), false, "Sword Down for Camera.001");
            },
            Event::Trigger { id: 16 } => {
                self.tri.thickness = self.cfg.f32("weight1");
            }

            Event::Trigger { id: 10 } => p.go("yume").await,

            Event::Mod { id: 0, fr } => *self.fx.alpha = fr,
            Event::Mod { id: 1, fr } => {
                self.fx.glitch = fr;
                self.fx.vhs = fr;
            },

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

        self.tri.encode(frame, self.composite.view(0));
        self.scene.encode(frame, self.composite.view(1));

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
