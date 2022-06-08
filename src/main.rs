#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::HashMap;

use anyhow::{Context, Result};
use lib::{prelude::*, window::WindowBuilder};

mod pipeline;
mod scenes;

mod demo;
use demo::{Demo, Player, Scene, Scenes};

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

    let mut scenes: HashMap<&'static str, Box<dyn Scene + Send>> = HashMap::new();
    scenes.insert("test_segments", Box::new(scenes::TestSegments::new(device)));
    scenes.insert("test1", Box::new(scenes::Test1::new(device)));
    scenes.insert("test2", Box::new(scenes::Test2::new(device)));

    let player = Player::new("ms7.dem", t0, "test_segments", scenes).expect("failed to load demo");

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

fn view(_app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.player.view(frame, view);
}
