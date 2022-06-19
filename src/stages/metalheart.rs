use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Metalheart {
    segment: Segment,

    t: f32,
    t_mul: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    tri: IsoTriPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
    Main
}

impl Metalheart {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("plonk", 0.0)
            .with("weight", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("metalheart.cfg");

        let scene = Phong::new(app, "metalheart.glb", |_node| true, |_mat| true);
        let tri = IsoTriPass::new(device, IsoTri {
            color: [0.369, 0.756, 0.871],
            aspect: 16.0 / 9.0,
            t: 0.0,
            r: -3.0,
            weight: cfg.f32("plonkweight"),
            thickness: 0.5,
        });

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "metalheart_composite",
            2,
            Some("composite.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("metalheart").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 0.3,

            decay,
            count,
            cfg,

            scene,
            tri,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Metalheart {
    async fn init(&mut self, p: &mut Player) {
        self.scene.light("Point").range = 0.0;
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += self.cfg.f32("tmul") * p.rms() * self.t_mul * dt;

        self.scene.node("Heart").transform.rotate = Euler {
            x: Rad(0.0),
            y: Rad(self.t * self.cfg.f32("speed")),
            z: Rad(0.0),
        }.into();

        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.tri.update(self.t);

        if let Segment::Init = self.segment {
            *self.fx.alpha = self.decay.v("plonk") * self.cfg.f32("plonkfr");
        }

        if let Segment::Main = self.segment {
            let min = self.cfg.f32("weightmin");
            let max = self.cfg.f32("weightmax");
            self.tri.thickness = min + self.decay.v("weight") * (max - min);
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;
        let cfg = &self.cfg;

        match ev {
            Event::Beat { id: 61, t } => self.decay.set_t("weight", t * 2.0),
            Event::Beat { id: 60, t } => self.decay.set_t("plonk", t * 2.0),

            Event::Mod { id: 1, fr } => self.fx.vhs = cfg.f32("vhs1") * fr,
            Event::Mod { id: 2, fr } => { self.fx.glitch = fr; self.fx.edge = fr; },
            Event::Mod { id: 3, fr } => self.fx.flash = fr,
            Event::Mod { id: 4, fr } => self.fx.vhs = cfg.f32("vhs2") * fr,

            Event::Trigger { id: 29 } => {
                self.segment = Segment::Main;
                *self.fx.alpha = 1.0;
                self.scene.light("Point").range = self.cfg.f32("lrange");
            },

            Event::Trigger { id: 10 } => p.go("cyber_grind").await,

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
