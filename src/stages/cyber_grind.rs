use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct CyberGrind {
    segment: Segment,

    t: f32,
    t_mul: f32,

    rot: f32,
    vel: f32,
    min: f32,
    accel: f32,
    acid: f32,
    edge: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    pyramid: Phong,
    core: Phong,
    ico: Phong,
    animator: Animator,

    starfield: StarfieldPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
    Drop,
    Rotate,
    RedFly,
    BlueFly,
    GreenFly,
}

impl CyberGrind {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("drop", 10.0)
            .with("stab", 0.0)
            .with("tap", 0.0)
            .with("clap", 0.0)
            .with("kick", 0.0)
            .with("womp", 0.0)
            ;
        let count = CounterEnv::default();

        let cfg = lib::resource::read_cfg("pyraship.cfg");

        let pyramid = Phong::new(app, "pyraship.glb", |_node| true, |_mat| true);
        let core = Phong::new(app, "coreship.glb", |_node| true, |_mat| true);
        let ico = Phong::new(app, "technoship.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&pyramid.scene);

        let starfield = StarfieldPass::new(device, Starfield {
            color: [1.0, 0.0, 0.0],
            x: 0.5,
            y: 0.85,
            w: 640.0,
            h: 360.0,
            t: 0.0,
            speed: 1.0,
            warp: cfg.f32("warp"),
            acid: 0.0,
        });

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "pyraship_composite",
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

            rot: 0.0,
            vel: 0.0,
            accel: cfg.f32("accel"),
            min: 0.0,
            acid: 0.0,
            edge: 0.0,

            decay,
            count,
            cfg,

            pyramid,
            core,
            ico,
            animator,

            starfield,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for CyberGrind {
    async fn init(&mut self, p: &mut Player) {
        // self.animator.play(self.t, true, "");
        // for i in 0..13 {
        //     for j in 0..13 {
        //         let n = i + j*12;

        //         let node = self.pyramid.node(&format!("Cube{:03}", n));
        //         // log::info!("Cube{:3} -> y {}", n, node.transform.translate.y);
        //     }
        // }

        // self.fx.bloom
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        let cfg = &self.cfg;

        self.t += 50.0 * p.rms() * self.t_mul * dt;

        self.animator.update(p.t(), &mut self.pyramid.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.starfield.update(p.t());

        let pyramid = self.pyramid.node("Pyramid");
        let core = self.core.node("Sphere");

        self.vel = max_partial!(self.min, self.vel - self.accel * dt);
        self.rot += self.vel * dt;
        let euler = Euler {
            x: Rad(0.0),
            y: Rad(self.rot),
            z: Rad(0.0),
        }.into();
        pyramid.transform.rotate = euler;
        core.transform.rotate = euler;
        self.ico.node("Icosphere").transform.rotate = euler;
        self.ico.node("IcosphereCore").transform.rotate = euler;

        let kick = self.decay.v("kick");
        pyramid.transform.scale = v3(2.0, 2.0, 2.0) + 0.5 * v3(kick, kick, kick);
        core.transform.scale = v3(3.0, 3.0, 3.0) + 0.5 * v3(kick, kick, kick);
        self.ico.node("Icosphere").transform.scale = v3(2.9, 2.9, 2.9) + 0.5 * v3(kick, kick, kick);
        self.ico.node("IcosphereCore").transform.scale = v3(2.5, 2.5, 2.5) + 0.5 * v3(kick, kick, kick);

        self.fx.mega = cfg.f32("mega") * self.decay.v("womp");
        self.fx.edge = self.decay.v("womp") + self.edge;
        self.fx.glitch = self.decay.v("womp");
        self.fx.shake = self.decay.v("clap");
        self.starfield.acid = min_partial!(1.0, self.acid + 0.5 * self.decay.v("womp"));

        if let Some(fr) = self.decay.vv("stab") {
            self.fx.flash = 0.6 * fr;
        }

        match self.segment {
            Segment::Init | Segment::Drop => {
                for i in 0..13 {
                    for j in 0..13 {
                        let n = i + j*13;

                        let drop = match self.decay.vv("drop") {
                            Some(fr) => 1.0 - fr,
                            None => 0.0,
                        } * 9.0;

                        let tap = self.decay.v("tap") * 0.15;

                        let sx = (i as f32 * 0.4 + self.t).sin() * 0.8;
                        let sy = (j as f32 * 0.5 + 5.0 + self.t).sin() * 0.8;

                        let node = self.pyramid.node(&format!("Cube{:03}", n));
                        node.transform.scale.y = 5.0 + (sx * sy);
                        node.transform.translate.y = -5.0 - drop - tap;
                    }
                }
            },
            _ => {}
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;
        let cfg = &self.cfg;

        match ev {
            Event::Beat { id: 64, t } => { decay.set_t("stab", t); decay.set_t("clap", t); },
            Event::Beat { id: 63, t } => decay.set_t("tap", t),
            Event::Beat { id: 62, t } => decay.set_t("clap", t),
            Event::Beat { id: 61, t } => decay.set_t("kick", t),
            Event::Beat { id: 60, t } => { decay.set_t("womp", t); self.vel += cfg.f32("boost") },

            Event::Trigger { id: 29 } => self.segment = match self.segment {
                Segment::Init => {
                    self.decay.set("drop");
                    Segment::Drop
                },
                Segment::Drop => {
                    self.animator.play(p.t(), false, "CameraAction");
                    for i in 0..13 {
                        for j in 0..13 {
                            let n = i + j*13;
                            let node = self.pyramid.node(&format!("Cube{:03}", n));
                            node.transform.translate.z = -5000.0;
                        }
                    }
                    Segment::Rotate
                },
                Segment::Rotate => {
                    self.min = self.cfg.f32("vmin");
                    Segment::RedFly
                },
                Segment::RedFly => {
                    self.starfield.color = [0.0, 0.0, 1.0];
                    Segment::BlueFly
                },
                Segment::BlueFly => {
                    self.starfield.color = [0.0, 1.0, 0.0];
                    Segment::GreenFly
                },
                Segment::GreenFly => {
                    Segment::GreenFly
                },
            },
            Event::Trigger { id: 10 } => p.go("halo").await,

            Event::Mod { id: 0, fr } => self.t_mul = 4.0 * fr,
            Event::Mod { id: 1, fr } => self.fx.red = fr,
            Event::Mod { id: 2, fr } => self.acid = fr,
            Event::Mod { id: 3, fr } => self.fx.invert = fr,
            Event::Mod { id: 4, fr } => { self.fx.vhs = fr; self.fx.glitch = fr; },
            Event::Mod { id: 5, fr } => self.fx.flash = fr,
            // Event::Mod { id: 6, fr } => self.starfield.warp = fr,

            Event::Mod { id: 7, fr } => self.edge = fr,
            Event::Mod { id: 8, fr } => *self.fx.alpha = 1.0 - fr,
            // Event::Mod { id: 8, fr } => self.fx.bloom = fr,

            // Event::Mod { id: 2, fr } => self.starfield.acid = fr,
            // Event::Mod { id: 3, fr } => self.starfield.speed = fr,
            // Event::Mod { id: 4, fr } => self.starfield.warp = fr,
            // Event::Mod { id: 5, fr } => {
            //     log::info!("y={:?}", fr);
            //     self.starfield.y = fr;
            // }
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
            Segment::RedFly | Segment::BlueFly | Segment::GreenFly =>
                self.starfield.encode(frame, self.composite.view(0)),
            _ => {}
        }

        match self.segment {
            Segment::BlueFly => self.core.encode(frame, self.composite.view(1)),
            Segment::GreenFly => self.ico.encode(frame, self.composite.view(1)),
            _ => self.pyramid.encode(frame, self.composite.view(1)),
        }

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}
