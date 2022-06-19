use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Resolve {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    wormhole: FlyTorusPass,

    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,

}

impl Resolve {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("kick", 0.0)
            .with("hat", 0.0)
            .with("snare", 0.0)
            .with("crash", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("resolve.cfg");

        let wormhole = FlyTorusPass::new(device, FlyTorus {
                color: [1.0, 0.1, 0.3],
                t: 0.0,
                speed: 0.3,
                mx: 16.0 / 9.0,
                my: 1.0,
                warp: 0.65,
        });

        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("resolve").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 0.5,

            decay,
            count,
            cfg,

            wormhole,

            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Resolve {
    async fn init(&mut self, p: &mut Player) {
        // self.animator.play(self.t, true, "");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.wormhole.update(self.t);

        self.wormhole.warp = 0.65 + 0.05 * self.decay.v("kick");

        self.fx.edge = self.decay.v("snare") * self.cfg.f32("edge");
        self.fx.shake = self.decay.v("snare") * self.cfg.f32("shake");
        self.fx.flash = self.decay.v("hat") * self.cfg.f32("flash");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 61, t } => self.decay.set_t("kick", t * 2.0),
            Event::Beat { id: 62, t } => self.decay.set_t("hat", t * 2.0),
            Event::Beat { id: 63, t } => self.decay.set_t("snare", t * 2.0),
            Event::Beat { id: 64, t } => self.decay.set_t("crash", t * 2.0),

            Event::Mod { id: 0, fr } => *self.fx.alpha = 1.0 - fr,

            Event::Trigger { id: 22 } => {
                self.fx.glitch = 0.0;
                self.fx.vhs = 0.0;
            },
            Event::Trigger { id: 23 } => {
                self.fx.glitch = 0.2;
                self.fx.vhs = 0.4;
            },
            Event::Trigger { id: 24 } => {
                self.fx.glitch = 0.1;
                self.fx.vhs = 0.3;
            },
            Event::Trigger { id: 25 } => {
                self.fx.glitch = 0.6;
                self.fx.vhs = 0.6;
            },
            Event::Trigger { id: 26 } => {
                self.fx.glitch = 0.9;
                self.fx.vhs = 0.9;
            },

            Event::Trigger { id: 21 } => self.t_mul = 2.0,
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

        self.wormhole.encode(frame, self.fx.view());

        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
