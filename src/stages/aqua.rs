use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Aqua {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator: Animator,

    clear: ClearPass,
    text: TextPass,
    tfx: FxPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
}

impl Aqua {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("crash", 0.0)
            .with("boost", 0.0)
            .with("headlight", 0.0)
            .with("bigkick", 0.0)
            .with("bigsnare", 0.0)
            .with("vhs", 0.0)
            ;
        let count = CounterEnv::default()
            .with("aqua", 1 + 1);

        let cfg = lib::resource::read_cfg("aqua.cfg");

        let scene = Phong::new(app, "aquanox.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);

        let clear = ClearPass::new(device, wgpu::Color::BLACK);
        let text = TextPassBuilder::default()
            .with("default", "da_mad_rave_italic.otf")
            .build(device);
        let mut tfx = FxPass::new(device, (640, 360));
        tfx.vhs = cfg.f32("tvhs");
        tfx.bloom = cfg.f32("tbloom");

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "aqua_composite",
            2,
            Some("composite_add.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("aqua").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            scene,
            animator,

            clear,
            text,
            tfx,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Aqua {
    async fn init(&mut self, p: &mut Player) {
        self.animator.play(self.t, true, "Landscape.001Action");
        self.animator.play(self.t, true, "Landscape.001Action.001");
        self.animator.play(self.t, true, "Landscape.002Action.001");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.animator.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.tfx.update(p.t(), self.t);

        self.fx.flash = self.decay.v("crash") * 0.5;

        if let Some(fr) = self.decay.vv("crash") {
            self.fx.flash = fr * 0.5;
        } else {
            self.fx.flash = self.decay.v("bigsnare") * self.cfg.f32("flash")
        }

        self.fx.shake = self.decay.v("bigkick") * self.cfg.f32("shake");

        self.fx.vhs = self.decay.v("vhs");

        // self.scene.light("Point.004").range = 40.0 * self.decay.v("bigsnare");
        // self.scene.light("Point.005").range = 40.0 * self.decay.v("bigsnare");
        // self.scene.light("Point").range = 40.0 * self.decay.v("bigkick");
        // self.scene.light("Point.001").range = 40.0 * self.decay.v("bigkick");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 64, t } => self.decay.set_t("crash", t),

            Event::Beat { id: 60, t } => decay.set_t("bigkick", t * 0.5),
            Event::Beat { id: 61, t } => decay.set_t("bigsnare", t),
            Event::Beat { id: 62, t } => decay.set_t("vhs", t),

            Event::Mod { id: 0, fr } => {
                self.fx.glitch = fr;
                self.fx.edge = fr;
            }

            Event::Trigger { id: 22 } => self.scene.node("Camera").transform = self.scene.node("Camera0").transform,
            Event::Trigger { id: 23 } => self.scene.node("Camera").transform = self.scene.node("Camera1").transform,
            Event::Trigger { id: 24 } => self.scene.node("Camera").transform = self.scene.node("Camera2").transform,
            Event::Trigger { id: 25 } => self.scene.node("Camera").transform = self.scene.node("Camera3").transform,
            Event::Trigger { id: 26 } => self.scene.node("Camera").transform = self.scene.node("Camera4").transform,
            Event::Trigger { id: 27 } => self.scene.node("Camera").transform = self.scene.node("Camera4").transform,

            Event::Trigger { id: 14 } => { self.fx.invert = 1.0 - self.fx.invert; },
            Event::Trigger { id: 13 } => { self.count.inc("aqua"); },

            Event::Trigger { id: 10 } => p.go("reality").await,

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

        if let Some(n) = self.count.vv("aqua") {
            let pos = cfg.v2("aqua");
            let scale = cfg.f32("aqua_s");
            let col = v4(1.0, 1.0, 1.0, 1.0);
            self.text.draw(|d| d.at(pos).text("aqua", |t| t.scale(scale).color(col)));

            self.text.encode(frame, self.tfx.view());
            self.tfx.upload(frame);
            self.tfx.encode(frame, self.composite.view(1));
        } else {
            self.clear.encode(frame, self.composite.view(1));
        }

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
