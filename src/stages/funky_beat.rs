use crate::util::*;
use async_trait::async_trait;
use lib::gfx::pass::*;
use lib::gfx::scene::Node;
use lib::prelude::*;

use crate::demo::{Event, Player, Stage};
use crate::pipeline::*;

pub struct FunkyBeat {
    segment: Segment,

    t: f32,
    t_mul: f32,

    animator: Animator,
    decay: DecayEnv,
    count: CounterEnv,
    cfg: Config,

    scene: Phong,

    text0: TextPass,
    text1: TextPass,
    tfx: FxPass,
    clear: ClearPass,

    composite: FilterPass,
    fx: FxPass,
    blit: BlitPass,
}

enum Segment {
    Intro,
    Synth,
    Twang,
    AhAh,
}

impl FunkyBeat {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let decay = DecayEnv::default()
            .with("crash", 0.0)
            .with("noise", 0.0)
            .with("rride", 0.0)
            .with("ride", 0.0)
            .with("bang", 0.0)
            .with("synth", 0.0)
            .with("hat", 0.0)
            .with("kick", 0.0)

            .with("boi", 0.0)
            .with("uh", 0.0)
            .with("ah", 0.0)
            .with("do", 0.0)
            .with("give", 0.0)
            .with("me", 0.0)
            .with("ow", 0.0)

            .with("edge", 0.0)
            .with("shake", 0.0)
            .with("glitch", 0.0)
            .with("vhs", 0.0)
            .with("pause", 0.0)
            .with("red", 0.0)
            .with("mega", 0.0)
            .with("invert", 0.0)
        ;

        let count = CounterEnv::default()
            .with("camjump", 5)
            .with("gimme", 7 + 1)
            .with("getup", 2 + 1)
            .with("getdown", 2 + 1)
            .with("ah", 8)
            .with("do", 8)
            .with("aint", 8 + 1)
            .with("sounds", 6 + 1)
            .with("fresh", 4 + 1)
            .with("boi", 3)
            .with("fun-ky", 2 + 1)
            .with("real", 7 + 1)
            .with("bock", 7 + 1)
            .with("cut", 6 + 1)
            ;

        let cfg = lib::resource::read_cfg("funky_beat.cfg");

        let text0 = TextPassBuilder::default()
            .with("default", "da_mad_rave_italic.otf")
            .build(device);
        let text1 = TextPassBuilder::default()
            .with("default", "da_mad_rave_italic.otf")
            .build(device);
        let mut tfx = FxPass::new(device, (640, 360));
        tfx.vhs = cfg.f32("vhs");
        tfx.bloom = cfg.f32("bloom");
        // tfx.edge = 1.0;
        let clear = ClearPass::new(device, wgpu::Color::BLACK);

        let scene = Phong::new(app, "cuberoom.glb", |_node| true, |_mat| true);
        let animator = Animator::new(&scene.scene);
        let composite = FilterPass::new_composite_sized::<()>(
            device,
            "funky",
            3,
            Some("composite_add.frag.spv"),
            None,
            (640, 360),
        );
        let fx = FxPass::new(device, (640, 360));
        let blit = BlitPass::new("funky").build(device);

        Self {
            segment: Segment::Intro,

            t: 0.0,
            t_mul: 1.0,

            animator,
            decay,
            count,
            cfg,

            text0,
            text1,
            tfx,
            clear,

            scene,
            composite,
            fx,
            blit,
        }
    }
}

#[async_trait]
impl Stage for FunkyBeat {
    async fn init(&mut self, p: &mut Player) {
        self.animator.play(self.t, true, "CubeAction");
    }

    async fn update(&mut self, p: &mut Player, dt: f32) {
        self.t += 50.0 * p.rms() * self.t_mul * dt;

        // Room white
        if let Some(fr) = self.decay.vv("hat") {
            let fr = fr * 0.025;
            self.scene.material("Room").color = v4(fr, fr, fr, fr);
        } else if let Some(fr) = self.decay.vv("kick") {
            self.scene.material("Room").color = v4(fr, fr, fr, fr);
        } else if let Some(fr) = self.decay.vv("crash") {
            self.scene.material("Room").color = v4(fr, fr, fr, fr);
        }

        // Cam jump
        self.scene.node("Camera.000").transform =
            self.scene.node(&format!("Camera.00{}", self.count.v("camjump"))).transform;

        // Cube big
        let scale = 0.5 + 0.05 * self.decay.v("synth");
        self.scene.node("Cube").transform.scale = v3(scale, scale, scale);

        // Crash white
        if let Some(fr) = self.decay.vv("crash") {
            self.fx.state.invert = fr;
        } else {
            self.fx.state.invert = 0.0;
        }

        // self.fx.state.edge = self.decay.v("edge");
        self.fx.state.edge = self.decay.v("ah");
        self.fx.state.flash = self.decay.v("rride");
        self.fx.state.vhs = match self.segment {
            Segment::Twang => self.decay.v("hat"),
            _ => 0.0,
        };

        self.fx.state.mega = self.decay.v("shake");
        self.fx.state.glitch = self.decay.v("glitch");
        self.fx.state.vhs = self.decay.v("vhs");
        self.fx.state.pause = self.decay.v("pause");
        self.fx.state.red = self.decay.v("red");
        self.fx.state.mega = self.decay.v("mega");
        self.fx.state.invert = self.decay.v("invert");

        self.animator.update(self.t, &mut self.scene.scene);
        self.decay.update(dt);
        self.fx.update(p.t(), self.t);
        self.tfx.update(p.t(), self.t);
    }

    async fn event(&mut self, p: &mut Player, ev: Event) {
        let decay = &mut self.decay;
        let count = &mut self.count;

        match ev {
            Event::Beat { id: 75, t } => { count.inc("do"); decay.set_t("do", t * 1.25); },
            Event::Beat { id: 74, t } => decay.set_t("ow", t * 2.0),
            Event::Beat { id: 73, t } => { count.inc("boi"); decay.set_t("boi", t * 2.0); },
            Event::Beat { id: 72, t } => {
                decay.set_t("ah", t * 1.25);
                decay.set_t("kick", t * 1.25);
                count.inc("ah");
                count.inc("camjump");
            },
            Event::Beat { id: 71, t } => decay.set_t("uh", t * 2.0),
            Event::Beat { id: 70, t } => decay.set_t("me", t * 2.0),
            Event::Beat { id: 69, t } => decay.set_t("give", t * 2.0),

            Event::Beat { id: 68, t } => decay.set_t("crash", t),
            Event::Beat { id: 67, t } => decay.set_t("noise", t * 4.0),
            Event::Beat { id: 66, t } => decay.set_t("rride", t),
            Event::Beat { id: 65, t } => decay.set_t("ride", t * 4.0),
            Event::Beat { id: 64, t } => decay.set_t("bang", t * 4.0),
            Event::Beat { id: 62, t } => decay.set_t("hat", t),
            Event::Beat { id: 61, t } => decay.set_t("synth", t),
            Event::Beat { id: 60, t } => { decay.set_t("kick", t * 1.25); count.inc("camjump"); },

            Event::Trigger { id: 25 } => self.segment = match self.segment {
                Segment::Intro => {
                    self.scene.material("Cube").color = v4(1.0, 1.0, 0.0, 1.0);
                    Segment::Synth
                },
                Segment::Synth => {
                    self.scene.material("Cube").color = v4(0.0, 1.0, 1.0, 1.0);
                    Segment::Twang
                },
                Segment::Twang => {
                    self.scene.material("Cube").color = v4(1.0, 0.0, 1.0, 1.0);
                    Segment::AhAh
                },
                Segment::AhAh => Segment::Intro,
            },
            Event::Trigger { id: 29 } => { count.inc("getup"); }
            Event::Trigger { id: 28 } => { count.inc("getdown"); }
            Event::Trigger { id: 27 } => { count.inc("gimme"); }
            Event::Trigger { id: 26 } => { count.inc("aint"); }
            Event::Trigger { id: 24 } => { count.inc("sounds"); },
            Event::Trigger { id: 23 } => { count.inc("fresh"); },
            Event::Trigger { id: 22 } => { count.inc("fun-ky"); },
            Event::Trigger { id: 21 } => { count.inc("real"); },
            Event::Trigger { id: 20 } => { count.inc("bock"); },
            Event::Trigger { id: 19 } => { count.inc("cut"); },

            Event::Trigger { id: 10 } => p.go("thanks").await,

            Event::Mod { id: 0, fr } => self.fx.state.invert = fr,
            Event::Mod { id: 1, fr } => self.t_mul = 2.0 * fr,

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

    #[rustfmt::skip]
    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        let text0 = &mut self.text0;
        let text1 = &mut self.text1;
        let decay = &self.decay;
        let count = &self.count;
        let cfg = &self.cfg;

        if let Some(fr) = decay.vv("noise") {
            text(text0, "*kssh*", 80.0, cfg.v2("noise"), v4(1.0, 1.0, 1.0, fr)); }
        if let Some(fr) = decay.vv("ride") {
            text(text0, "*ting*", 80.0, cfg.v2("ride"), v4(1.0, 1.0, 1.0, fr)); }
        if let Some(fr) = decay.vv("bang") {
            text(text0, "*bang*", 80.0, cfg.v2("bang"), v4(1.0, 1.0, 1.0, fr)); }

        if let Some(fr) = decay.vv("give") {
            text(text0, "give", cfg.f32("give_s"), cfg.v2("give"), v4(1.0, 1.0, 1.0, fr)); }
        if let Some(fr) = decay.vv("me") {
            text(text0, "me", cfg.f32("me_s"), cfg.v2("me"), v4(1.0, 1.0, 1.0, fr)); }

        if let Some(fr) = decay.vv("boi") {
            text(text0, "boi!", cfg.f32("boi_s"), cfg.v2(&format!("boi{}", count.v("boi"))), v4(1.0, 1.0, 1.0, fr)); }
        if let Some(fr) = decay.vv("uh") {
            text(text0, "uh!", cfg.f32("uh_s"), cfg.v2("uh"), v4(1.0, 1.0, 1.0, fr)); }
        if let Some(fr) = decay.vv("ow") {
            text(text0, "ow!", cfg.f32("ow_s"), cfg.v2("ow"), v4(1.0, 1.0, 1.0, fr)); }

        if let Some(fr) = decay.vv("ah") {
            text(text0, "ah!", cfg.f32("ah_s"), cfg.v2(&format!("ah{}", count.v("ah"))), v4(1.0, 0.0, 1.0, 1.0)) }

        if let Some(fr) = decay.vv("do") {
            text(text0, "do!", cfg.f32("do_s"), cfg.v2(&format!("do{}", count.v("do"))), v4(1.0, 1.0, 0.0, 1.0)) }


        let w = v4(1.0, 1.0, 1.0, 1.0);
        let fw = v4(1.0, 1.0, 1.0, cfg.f32("fade"));

        type_text(text0, count.v("getup"), "get^ up^", cfg.f32("getup_s"), cfg.v2("getup"), w);
        type_text(text0, count.v("getdown"), "get^ down^", cfg.f32("getdown_s"), cfg.v2("getdown"), fw);

        place_texts(count.v("gimme"), &mut [text0, text1], &[
            (0, vec![
                ("gim",   cfg.f32("gimmes0"), cfg.v2("gimme0"), fw),
                ("gimme", cfg.f32("gimmes1"), cfg.v2("gimme1"), fw),
                ("one",   cfg.f32("gimmes2"), cfg.v2("gimme2"), fw),
                ("dem",   cfg.f32("gimmes3"), cfg.v2("gimme3"), fw),
            ]),
            (1, vec![
                ("fun",   cfg.f32("funkys"), cfg.v2("funky"), w),
                ("funky", cfg.f32("funkys"), cfg.v2("funky"), w),
                ("beats", cfg.f32("beats"), cfg.v2("beats0"), w),
            ]),
        ]);

        place_texts(count.v("aint"), &mut [text0, text1], &[
            (0, vec![
                ("ain't",   cfg.f32("aints0"), cfg.v2("aint0"), fw),
                ("no",      cfg.f32("aints1"), cfg.v2("aint1"), fw),
                ("nothing", cfg.f32("aints2"), cfg.v2("aint2"), fw),
                ("like",    cfg.f32("aints3"), cfg.v2("aint3"), fw),
                ("a",       cfg.f32("aints4"), cfg.v2("aint4"), fw),
            ]),
            (1, vec![
                ("fun",   cfg.f32("funkys"), cfg.v2("funky"), w),
                ("funky", cfg.f32("funkys"), cfg.v2("funky"), w),
                ("beat",  cfg.f32("beats"), cfg.v2("beat0"), w),
            ]),
        ]);

        place_texts(count.v("sounds"), &mut [text0, text1], &[
            (0, vec![
                ("sounds", cfg.f32("soundss0"), cfg.v2("sounds0"), fw),
                ("so",     cfg.f32("soundss1"), cfg.v2("sounds1"), fw),
            ]),
            (1, vec![
                ("fun",   cfg.f32("funkys"), cfg.v2("funky"), w),
                ("funky", cfg.f32("funkys"), cfg.v2("funky"), w),
            ]),
            (0, vec![
                ("to", cfg.f32("soundss2"), cfg.v2("sounds2"), w),
                ("to me", cfg.f32("soundss2"), cfg.v2("sounds2"), w),
            ]),
        ]);

        place_texts(count.v("fresh"), &mut [text0, text1], &[
            (1, vec![
                ("fun",   cfg.f32("funkys"), cfg.v2("funky"), w),
                ("funky", cfg.f32("funkys"), cfg.v2("funky"), w),
                ("fresh", cfg.f32("freshs0"), cfg.v2("fresh0"), fw),
                ("beats", cfg.f32("freshs1"), cfg.v2("fresh1"), fw),
            ]),
        ]);

        place_text(text1, count.v("fun-ky"), &[
            ("fun", cfg.f32("fun-kys0"), cfg.v2("fun-ky0"), w),
            ("ky",  cfg.f32("fun-kys1"), cfg.v2("fun-ky1"), w),
        ]);

        place_text(text0, count.v("real"), &[
            ("ain't",   cfg.f32("reals0"), cfg.v2("real0"), fw),
            ("no",      cfg.f32("reals1"), cfg.v2("real1"), fw),
            ("nothing", cfg.f32("reals2"), cfg.v2("real2"), fw),
            ("like",    cfg.f32("reals3"), cfg.v2("real3"), fw),
            ("the",     cfg.f32("reals4"), cfg.v2("real4"), fw),
            ("real",    cfg.f32("reals5"), cfg.v2("real5"), fw),
            ("thing",   cfg.f32("reals6"), cfg.v2("real6"), fw),
        ]);

        place_texts(count.v("bock"), &mut [text0, text1], &[
            (0, vec![
                ("bock", cfg.f32("bocks0"), cfg.v2("bock0"), fw),
                ("the",  cfg.f32("bocks1"), cfg.v2("bock1"), fw),
                ("bock", cfg.f32("bocks2"), cfg.v2("bock2"), fw),
                ("i'm",  cfg.f32("bocks3"), cfg.v2("bock3"), fw),
                ("on",   cfg.f32("bocks4"), cfg.v2("bock4"), fw),
            ]),
            (1, vec![
                ("fun", cfg.f32("fun-kys0"), cfg.v2("fun-ky0"), w),
                ("ky",  cfg.f32("fun-kys1"), cfg.v2("fun-ky1"), w),
            ]),
        ]);

        place_texts(count.v("cut"), &mut [text0, text1], &[
            (0, vec![
                ("cut",  cfg.f32("cuts0"), cfg.v2("cut0"), fw),
                ("the",  cfg.f32("cuts1"), cfg.v2("cut1"), fw),
                ("beat", cfg.f32("cuts2"), cfg.v2("cut2"), fw),
                ("it's", cfg.f32("cuts3"), cfg.v2("cut3"), fw),
            ]),
            (1, vec![
                ("fun", cfg.f32("fun-kys0"), cfg.v2("fun-ky0"), w),
                ("ky",  cfg.f32("fun-kys1"), cfg.v2("fun-ky1"), w),
            ]),
        ]);

        self.scene.encode(frame, self.composite.view(0));

        self.clear.encode(frame, self.composite.view(1));
        self.text0.encode(frame, self.composite.view(1));

        self.clear.encode(frame, self.tfx.view());
        self.text1.encode(frame, self.tfx.view());
        self.tfx.upload(frame);
        self.tfx.encode(frame, self.composite.view(2));

        self.composite.encode(frame, self.fx.view());
        self.fx.upload(frame);
        self.fx.encode(frame, self.blit.view());
        self.blit.encode(frame, view);
    }
}

fn text(pass: &mut TextPass, s: &str, scale: f32, pos: Vector2, color: Vector4) {
    pass.draw(|d| d.at(pos).text(s, |t| t.scale(scale).color(color)));
}

fn type_text(pass: &mut TextPass, n: usize, s: &str, scale: f32, pos: Vector2, color: Vector4) {
    match n {
        n if n > 0 => {
            let i = s.match_indices("^").nth(n - 1).expect("no space match").0;
            let s = s.split_at(i).0.replace("^", "");
            text(pass, &s, scale, pos, color)
        },
        _ => {},
    }
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

pub fn place_texts(n: usize, passes: &mut [&mut TextPass], ss: &[(usize, Vec<(&str, f32, Vector2, Vector4)>)]) {
    match n {
        0 => {},
        n => {
            let n = n as i32;
            let mut i0 = 0;
            let mut i1 = 0;
            for (pass, s) in ss {
                i0 += s.len() as i32;
                if i0 >= n as i32 && i1 <= n {
                    place_text(passes[*pass], (n - i1) as usize, s);
                }
                i1 += s.len() as i32;
            }
        }
    }
}
