use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Chaos {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    digits: DigitsPass,

    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,

}

impl Chaos {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("kick", 0.0)
            .with("snare", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("chaos.cfg");

        let digits = DigitsPass::new(device, [1.0, 0.1, 0.3]);

        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("chaos").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            digits,

            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Chaos {
    async fn init(&mut self, p: &mut Player) {
        *self.fx.alpha = 0.0;
        self.fx.edge = self.cfg.f32("edge");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.digits.update(self.t);

        let min = self.cfg.f32("edge");
        self.fx.edge = min + (1.0 - min) * self.decay.v("kick");
        self.fx.shake = self.decay.v("kick") * self.cfg.f32("shake");
        self.fx.flash = self.decay.v("snare") * self.cfg.f32("flash");
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 60, t } => self.decay.set_t("kick", t * 0.5),
            Event::Beat { id: 61, t } => {
                self.decay.set_t("snare", t);
                self.digits.permute();
            }

            Event::Mod { id: 0, fr } => *self.fx.alpha = fr,

            Event::Trigger { id: 10 } => p.go("dragonage").await,
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

        self.digits.encode(frame, self.fx.view());

        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
