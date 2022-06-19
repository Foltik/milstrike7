use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;
use palette::{Hsl, Srgb, FromColor};

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct Reality {
    segment: Segment,

    t: f32,
    t_mul: f32,
    rot: f32,
    rot_speed: f32,

    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,
    animator1: Animator,
    animator2: Animator,
    scale: f32,
    flash: f32,

    clear: ClearPass,
    text: TextPass,
    tfx: FxPass,

    spiral: SpiralPass,
    spiralamt: f32,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Init,
    Spiral,
    Rainbow,
}

impl Reality {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("hat", 0.0)
            .with("beep", 0.0)
            .with("synth", 0.0)
            .with("bam", 0.0)
            .with("bamshake", 0.0)
            .with("spiralbeat", 0.0)
            ;
        let count = CounterEnv::default()
            .with("getdown", 3)
            ;

        let cfg = lib::resource::read_cfg("reality.cfg");

        let scene = Phong::new(app, "uvbounce.glb", |_node| true, |_mat| true);
        let animator1 = Animator::new(&scene.scene);
        let animator2 = Animator::new(&scene.scene);

        let clear = ClearPass::new(device, wgpu::Color::BLACK);
        let text = TextPassBuilder::default()
            .with("default", "da_mad_rave_italic.otf")
            .build(device);
        let mut tfx = FxPass::new(device, (640, 360));
        tfx.vhs = cfg.f32("tvhs");
        tfx.bloom = cfg.f32("tbloom");

        let spiral = SpiralPass::new(device, Spiral {
            color: [1.0, 1.0, 1.0],
            aspect: 360.0 / 640.0,
            t: 0.0,
            swirl: 0.0,
            speed: 0.5,
            cutoff: 0.7,
            amount: 0.0,
            spokes: 3,
        });

        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "reality",
            3,
            Some("composite_add.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("reality").build(device);

        Self {
            segment: Segment::Init,

            t: 0.0,
            t_mul: 1.0,
            rot: 0.0,
            rot_speed: 0.0,

            decay,
            count,
            cfg,

            scene,
            animator1,
            animator2,
            scale: 0.0,
            flash: 0.0,

            clear,
            text,
            tfx,

            spiral,
            spiralamt: 0.0,

            composite,
            fx,
            blit
        }
    }
}

#[async_trait]
impl Stage for Reality {
    async fn init(&mut self, p: &mut Player) {
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 50.0 * p.rms() * self.t_mul * dt;

        self.animator1.update(p.t(), &mut self.scene.scene);
        self.animator2.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.tfx.update(p.t(), self.t);
        self.fx.update(p.t(), self.t);

        self.scene.light("PointTop").range = 2.0 + self.decay.v("hat");

        let s = 0.3 + self.decay.v("synth") * 0.1 * self.scale;
        self.scene.node("Sphere").transform.scale = v3(s, s, s);

        self.rot += self.rot_speed * 50.0 * p.rms() * self.t_mul * dt;
        self.scene.node("Sphere").transform.rotate = Euler {
            x: Rad(0.0),
            y: Rad(self.rot),
            z: Rad(0.0)
        }.into();

        self.spiral.t = self.t;
        self.spiral.amount = self.decay.v("spiralbeat") + self.spiralamt;

        self.fx.flash = self.decay.v("bam") + self.flash;
        self.fx.shake = self.decay.v("bamshake");

        if let Segment::Rainbow = self.segment {
            let hsl = Hsl::new((self.t / 4.0) * 360.0 % 360.0, 1.0, 0.5);
            let rgb = Srgb::from_color(hsl);
            self.scene.material("SphereChecker").color = v4(rgb.red, rgb.green, rgb.blue, 1.0);
            self.scene.material("SphereWhite").color = v4(rgb.red, rgb.green, rgb.blue, 1.0);
        }
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;
        let cfg = &self.cfg;

        match ev {
            Event::Beat { id: 65, t } => decay.set_t("spiralbeat", t * 0.75),
            Event::Beat { id: 64, t } => {
                decay.set_t("bam", t * 2.0);
                decay.set_t("bamshake", t);
            },
            Event::Beat { id: 63, t } => decay.set_t("synth", t * 2.0),
            Event::Beat { id: 62, t } => decay.set_t("hat", t * 2.0),
            Event::Beat { id: 61, t } => decay.set_t("beep", t * 2.0),

            // Event::Mod { id: 0, fr } => self.spiral.cutoff = fr,
            Event::Mod { id: 0, fr } => self.scale = fr * 5.0,
            Event::Mod { id: 1, fr } => self.flash = fr,
            Event::Mod { id: 2, fr } => {
                self.fx.glitch = fr;
                self.fx.vhs = fr;
            },
            // Event::Mod { id: 3, fr } => self.spiral.spokes = (8.0 * fr).floor() as u32,

            Event::Trigger { id: 28 } => self.animator1.play(p.t(), false, "SphereFall"),
            Event::Trigger { id: 27 } => self.rot_speed = cfg.f32("rot_speed"),
            Event::Trigger { id: 26 } => {
                self.scene.light("PointL").range = 2.0;
                self.scene.light("PointR").range = 2.0;
                log::info!("Sphere tr {:?}", self.scene.node("Sphere").transform.translate);
                self.animator2.play(self.t, true, "Figure8");
            },
            Event::Trigger { id: 25 } => { self.count.inc("getdown"); },
            Event::Trigger { id: 24 } => {
                let i = self.scene.mat_names["SphereChecker"];
                let j = self.scene.mat_names["SphereWhite"];
                self.scene.mats.swap(i, j);
                self.segment = Segment::Rainbow;
                self.fx.edge = cfg.f32("edge1");
                self.spiral.color = [1.0, 1.0, 1.0];
                self.spiral.spokes = 6;
            },
            Event::Trigger { id: 23 } => {
                self.animator2.stop("Figure8");
                self.scene.node("Cube").transform.translate.y = -1000.0;
                self.scene.node("Sphere").transform.translate = v3(0.0, 0.606, 1.7801435);
                self.segment = Segment::Spiral;
            },

            Event::Trigger { id: 21 } => {
                self.fx.edge = cfg.f32("edge1");
                self.spiral.color = [0.8, 0.0, 0.0];
                self.spiralamt = 1.0;
                self.spiral.swirl = 1.0;
                self.spiral.spokes = 5;
                self.spiral.speed = 3.0;
            },
            Event::Trigger { id: 20 } => {
                self.fx.edge = cfg.f32("edge2");
                self.spiral.color = [0.8, 0.0, 0.8];
                self.spiral.spokes = 8;
                self.spiral.speed = 10.0;
            }

            Event::Trigger { id: 10 } => p.go("pod").await,

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
                self.scene.encode(frame, self.composite.view(0));
            },
            Segment::Spiral | Segment::Rainbow => {
                if let Some(n) = self.count.vv("getdown") {
                    let col = v4(1.0, 1.0, 1.0, 1.0);
                    place_text(&mut self.text, n, &[
                        ("get", cfg.f32("get_sz"), cfg.v2("get"), col),
                        ("down",  cfg.f32("down_sz"), cfg.v2("down"), col),
                    ]);

                    self.text.encode(frame, self.tfx.view());
                    self.tfx.upload(frame);
                    self.tfx.encode(frame, self.composite.view(2));
                } else {
                    self.clear.encode(frame, self.composite.view(2));
                }

                self.spiral.encode(frame, self.composite.view(0));
                self.scene.encode(frame, self.composite.view(1));
            },
        }

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}

fn text(pass: &mut TextPass, s: &str, scale: f32, pos: Vector2, color: Vector4) {
    pass.draw(|d| d.at(pos).text(s, |t| t.scale(scale).color(color)));
}

fn place_text(pass: &mut TextPass, n: usize, s: &[(&str, f32, Vector2, Vector4)]) {
    match n {
        0 => {},
        n => {
            for (t, scale, pos, color) in s.iter().take(n) {
                text(pass, t, *scale, *pos, *color);
            }
        }
    }
}
