use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;
use palette::{Hsl, Srgb, FromColor};

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Lobby {
    segment: Segment,

    t: f32,
    t_mul: f32,

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

impl Lobby {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("bling", 0.0)
            .with("hat", 0.0)
            .with("hat2", 0.0)
            .with("kick", 0.0)
            .with("snare", 0.0)
            ;

        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("lobby.cfg");

        let scene = Phong::new(app, "demo_console.glb", |_node| true, |_mat| true);
        let animator1 = Animator::new(&scene.scene);
        let animator2 = Animator::new(&scene.scene);

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "lobby_composite",
            2,
            Some("composite.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("funky").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

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
impl Stage for Lobby {
    async fn init(&mut self, p: &mut Player) {
        *self.fx.alpha = 0.0;
        self.animator1.play(p.t(), true, "Idle Disc Bob Loop");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.animator1.update(p.t(), &mut self.scene.scene);
        self.animator2.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);

        let hsl = Hsl::new(0.0, self.decay.v("hat"), 0.5);
        let rgb = Srgb::from_color(hsl);
        self.scene.material("ButtonMain").color = v4(rgb.red, rgb.green, rgb.blue, 1.0);
        self.scene.light("ButtonMainPoint").range = self.cfg.f32("lrange") * self.decay.v("hat2");

        self.fx.shake = self.decay.v("kick");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            // Event::Beat { id: 60, t } => self.decay.set_t("kick", t * 2.0),
            Event::Beat { id: 61, t } => {
                self.decay.set_t("hat", t * 2.0);
                self.decay.set_t("hat2", t);
            },
            Event::Beat { id: 62, t } => self.decay.set_t("kick", t),
            Event::Beat { id: 63, t } => self.decay.set_t("snare", t),

            Event::Trigger { id: 29 } => self.animator2.play(self.t, true, "Idle Disc Rotate Loop"),
            Event::Trigger { id: 28 } => {
                self.animator1.stop("Idle Disc Bob Loop");
                self.animator1.play(p.t(), false, "Disc Insert");
            },
            Event::Trigger { id: 10 } => p.go("metalheart").await,

            Event::Mod { id: 0, fr } => self.t_mul = fr * 2.0,
            Event::Mod { id: 1, fr } => *self.fx.alpha = 1.0 - fr,
            Event::Mod { id: 2, fr } => self.fx.flash = fr,
            Event::Mod { id: 3, fr } => self.fx.edge = fr,
            Event::Mod { id: 4, fr } => self.fx.vhs = fr,

            _ => {}
        }
    }

    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            Key::Key1 => { log::info!("Disc rotate"); self.animator2.play(self.t, false, "Idle Disc Rotate Loop"); },
            Key::Key2 => { log::info!("Disc Insert"); self.animator1.play(p.t(), false, "Disc Insert"); },
            _ => {}
        }
    }

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        let decay = &self.decay;
        let count = &self.count;
        let cfg = &self.cfg;

        self.scene.encode(frame, self.composite.view(1));

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
