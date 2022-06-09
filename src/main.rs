#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::HashMap;

use anyhow::{Context, Result};
use lib::{prelude::*, window::WindowBuilder};

mod pipeline;
mod stages;

mod demo;
use demo::{Demo, Player, Stage, Stages};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        lib::app::run(window, model, input, update, view)?;
    } else {
        let audio_file = &args[1];
        let midi_file = &args[2];
        let demo_file = "resources/demos/ms7.dem";
        Demo::new(audio_file, midi_file)?.save(demo_file)?;
    }

    Ok(())
}

pub struct Model {
    player: Player,
}

fn window(mut window: WindowBuilder) -> WindowBuilder {
    window = window.title("Millenium Strike 7");

    if let Some(_) = std::env::args().find(|arg| arg.starts_with("-f")) {
        window = window.fullscreen();
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
    stages.insert("test", Box::new(stages::Test::new(app)));
    stages.insert("test_segments", Box::new(stages::TestSegments::new(device)));
    stages.insert("test1", Box::new(stages::Test1::new(device)));
    stages.insert("test2", Box::new(stages::Test2::new(device)));
    stages.insert("cyber_grind", Box::new(stages::CyberGrind::new(app)));
    stages.insert("funky_beat", Box::new(stages::FunkyBeat::new(device)));

    let scene0 = "test";
    let player = Player::new("ms7.dem", t0, scene0, stages).expect("failed to load demo");

    Model { player }
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
    m.player.update(dt).await;
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, depth: &wgpu::RawTextureView, target: &wgpu::RawTextureView) {
    m.player.view(frame, depth, target);
}
