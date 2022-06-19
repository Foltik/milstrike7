#![feature(exclusive_range_pattern)]
#![feature(stmt_expr_attributes)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::HashMap;

use anyhow::{Context, Result};
use lib::{prelude::*, window::WindowBuilder, midi2::device::worlde_easycontrol9::{WorldeEasyControl9, Input as MidiInput}};

mod pipeline;
mod stages;
mod util;

mod demo;
use demo::{Demo, Player, Stage, Stages};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().find(|arg| arg.starts_with("--render")) {
        None => lib::app::run(window, model, input, update, view)?,
        Some(_) => {
            let audio_file = &args[2];
            let midi_file = &args[3];
            let demo_file = "resources/demos/ms7.dem";
            Demo::new(audio_file, midi_file)?.save(demo_file)?;
        }
    }

    Ok(())
}

pub struct Model {
    player: Player,
    midi: Option<Midi<WorldeEasyControl9>>,
}

fn window(mut window: WindowBuilder) -> WindowBuilder {
    window = window.title("Millenium Strike 7");

    if let Some(_) = std::env::args().find(|arg| arg.starts_with("-w")) {
        // windowed
    } else {
        window = window.fullscreen_borderless();
    }

    window
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let t0 = match std::env::args().find(|arg| arg.starts_with("--t0=")) {
        Some(arg) => match arg.split('=').collect_tuple() {
            Some((_, t0)) => t0.parse::<f32>().unwrap(),
            _ => 0.0,
        },
        None => 0.0,
    };

    let mut stages: HashMap<&'static str, Box<dyn Stage + Send>> = HashMap::new();

    // DONE
    stages.insert("lobby", Box::new(stages::Lobby::new(app)));

    // DONE
    stages.insert("metalheart", Box::new(stages::Metalheart::new(app)));

    // DONE
    stages.insert("cyber_grind", Box::new(stages::CyberGrind::new(app)));

    // DONE
    stages.insert("halo", Box::new(stages::Halo::new(app)));

    // TODO
    stages.insert("aqua", Box::new(stages::Aqua::new(app)));

    // DONE
    stages.insert("reality", Box::new(stages::Reality::new(app)));

    // DONE
    stages.insert("pod", Box::new(stages::Pod::new(app)));

    // TODO
    stages.insert("chaostheory", Box::new(stages::Chaos::new(app)));

    // INPROGRESS
    stages.insert("dragonage", Box::new(stages::Dragon::new(app)));

    // TODO
    stages.insert("yume", Box::new(stages::Yume::new(app)));
    stages.insert("resolve", Box::new(stages::Resolve::new(app)));

    // DONE
    stages.insert("funky_beat", Box::new(stages::FunkyBeat::new(app)));

    // TODO
    stages.insert("thanks", Box::new(stages::Thanks::new(app)));

    // let scene0 = "lobby";
    let scene0 = "lobby";
    let player = Player::new("ms7.dem", t0, scene0, stages).expect("failed to load demo");

    let midi = Midi::<WorldeEasyControl9>::maybe_open("WORLDE easy control", "WORLDE easy control");

    Model { player, midi }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {
    if state != KeyState::Pressed {
        return;
    }

    match key {
        Key::Space => m.player.play(),
        Key::Q => app.exit(),
        _ => m.player.key(state, key).await,
    }
}

async fn update(app: &App, m: &mut Model, dt: f32) {
    if let Some(midi) = m.midi.as_mut() {
        for ev in midi.recv() {
            match ev {
                MidiInput::Slider(id, fr) => m.player.trigger(demo::Event::Mod { id, fr }).await,
                _ => {}
            }
        }
    }

    m.player.update(dt).await;
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, target: &wgpu::RawTextureView) {
    m.player.view(frame, target);
}
