use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Pod {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator: Animator,

    white: SynthPass,
    starfield: StarfieldPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
    Fast,
}

impl Pod {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("bigkick", 0.0)
            .with("bigsnare", 0.0)
            .with("noise", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("pod.cfg");

        let scene = Phong::new(app, "oceanfly.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);

        let white = SynthPass::new::<()>(device, "white", "white.frag.spv", None);
        let starfield = StarfieldPass::new(device, Starfield {
            color: [0.8, 0.3, 0.0],
            x: 0.5,
            y: cfg.f32("star_y"),
            w: 640.0,
            h: 360.0,
            t: 0.0,
            speed: cfg.f32("speed"),
            warp: cfg.f32("warp"),
            acid: 0.0,
        });

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "pod_composite",
            2,
            Some("composite.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("pod").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            scene,
            animator,

            white,
            starfield,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Pod {
    async fn init(&mut self, p: &mut Player) {
        self.animator.play(self.t, true, "PovFly");
        self.animator.play(self.t, true, "SunFly");
        // self.animator.play(self.t, true, "LightFly");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.animator.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.starfield.update(self.t);

        self.fx.shake = self.decay.v("bigkick") * self.cfg.f32("shake");

        if let Some(fr) = self.decay.vv("noise") {
            self.fx.flash = fr * self.cfg.f32("noise");
        } else {
            self.fx.flash = self.decay.v("bigsnare") * self.cfg.f32("flash");
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 60, t } => decay.set_t("bigkick", t * 0.5),
            Event::Beat { id: 61, t } => decay.set_t("bigsnare", t),

            Event::Beat { id: 62, t } => decay.set_t("bigkick", t * 2.0),
            Event::Beat { id: 63, t } => decay.set_t("bigsnare", t * 4.0),

            Event::Beat { id: 64, t } => decay.set_t("noise", t * 2.0),

            Event::Mod { id: 0, fr } => self.fx.glitch = fr,
            Event::Mod { id: 1, fr } => self.fx.vhs = fr,
            // Event::Mod { id: 2, fr } => self.fx.edge = fr,

            Event::Mod { id: 2, fr } => *self.fx.alpha = 1.0 - fr,

            Event::Trigger { id: 29 } => {
                self.fx.edge = self.cfg.f32("edge");
                self.t_mul = self.cfg.f32("tmul2");
                self.segment = Segment::Fast;
            },

            Event::Trigger { id: 13 } => self.fx.invert = 1.0 - self.fx.invert,


            Event::Trigger { id: 10 } => p.go("chaostheory").await,

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

        match self.segment {
            Segment::Init => {
                self.white.encode(frame, self.composite.view(0));
                self.scene.encode(frame, self.composite.view(1));
            },
            Segment::Fast => {
                self.starfield.encode(frame, self.composite.view(0));
                self.scene.encode(frame, self.composite.view(1));
            },
        }

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
