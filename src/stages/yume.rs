use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;
use lib::time::Spring;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Yume {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    waves: LineWavePass,
    spring: Spring,

    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,

}

impl Yume {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("crash", 0.0)
            .with("synth", 0.0)
            .with("kick", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("yume.cfg");

        let waves = LineWavePass::new(device, LineWave {
            color: [0.25, 0.78, 1.0],
            t: 0.0,
            w: 640.0,
            h: 360.0,
            n1: 0.0,
            n2: 0.48,
            dz: 0.25,
            thickness: 0.25,
            falloff: 0.07,
            n: cfg.usize("n") as u32,
        });
        let spring = Spring::new(1000.0);

        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("yume").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,

            decay,
            count,
            cfg,

            waves,
            spring,

            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Yume {
    async fn init(&mut self, p: &mut Player) {
        // self.animator.play(self.t, true, "");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.waves.update(self.t);

        // self.spring.set(self.cfg.f32("scale") * 100.0 * p.rms());
        // self.spring.update(dt);
        // self.waves.n1 = self.spring.v();
        self.waves.n1 = 0.4 * self.decay.v("synth");
        self.waves.thickness = 0.25 + 0.75 * self.decay.v("kick");
        self.waves.dz = self.cfg.f32("dz") + self.cfg.f32("ddz") * self.decay.v("kick");

        self.fx.edge = self.decay.v("synth") * self.cfg.f32("edge");
        self.fx.shake = self.decay.v("kick") * self.cfg.f32("shake");

        if let Some(fr) = self.decay.vv("crash") {
            self.fx.flash = 0.6 * fr;
        } else {
            self.fx.flash = self.decay.v("kick") * self.cfg.f32("flash");
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 69, t } => self.decay.set_t("crash", t * 2.0),

            Event::Beat { id: 66, t } => self.decay.set_t("synth", t * 2.0),
            Event::Beat { id: 61, t } => self.decay.set_t("kick", t * 2.0),

            Event::Trigger { id: 10 } => p.go("resolve").await,
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

        self.waves.encode(frame, self.fx.view());

        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
