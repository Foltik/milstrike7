#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use lib::{prelude::*, window::WindowBuilder};
use anyhow::{Result, Context};

mod pipeline;
mod scenes;

mod demo; use demo::{Player, Demo};
mod dispatch; use dispatch::Dispatch;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        lib::app::run(window, model, input, update, view)?;
    } else {
        Demo::new(&args[1], &args[2])?.save("resources/demos/ms7.dem")?;
    }

    Ok(())
}

pub struct Model {
    player: Player,
    dispatch: Option<Dispatch>,
}

impl Model {
    pub fn t(&self) -> f32 {
        self.player.t()
    }

    pub fn rms(&self) -> f32 {
        self.player.rms()
    }
}

fn window(window: WindowBuilder) -> WindowBuilder {
    window.title("Millenium Strike 7")
        // .fullscreen()
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let args: Vec<String> = std::env::args().collect();
    let start: f32 = if args.len() == 1 {
        0.0
    } else {
        args[1].parse().unwrap()
    };

    Model {
        player: Player::new("ms7.dem", start).expect("failed to load demo"),
        dispatch: Some(Dispatch::new(device, vec![
            Box::new(scenes::Test::new(device))
        ]))
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {
    if state != KeyState::Pressed { return; }

    match key {
        Key::Space => m.player.play(),
        Key::Q => app.exit(),
        _ => {}
    }
}

async fn update(app: &App, m: &mut Model, dt: f32) {
    m.player.update(dt);

    let events = m.player.events().collect::<Vec<_>>();
    for ev in events {
        log::debug!("Event: {:?}", ev);
        m.dispatch = Some(m.dispatch.take().unwrap().midi(app, m, ev).await);
    }
    
    m.dispatch = Some(m.dispatch.take().unwrap().update(app, m, dt).await);
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.dispatch = Some(m.dispatch.take().unwrap().encode(frame, view));
}
