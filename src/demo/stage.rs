use std::collections::HashMap;

use async_trait::async_trait;
use lib::prelude::*;

use super::{Event, Player};

#[async_trait]
pub trait Stage {
    async fn init(&mut self, p: &mut Player);
    async fn update(&mut self, p: &mut Player, dt: f32);

    async fn event(&mut self, p: &mut Player, ev: Event);
    async fn key(&mut self, p: &mut Player, state: KeyState, key: Key);

    fn view(&mut self, frame: &mut Frame, depth: &wgpu::RawTextureView, target: &wgpu::RawTextureView);
}

pub struct Stages {
    current: &'static str,
    scenes: HashMap<&'static str, Box<dyn Stage + Send>>,
    first_init: bool,
}

impl Stages {
    pub fn new(
        initial: &'static str,
        scenes: HashMap<&'static str, Box<dyn Stage + Send>>,
    ) -> Self {
        Self {
            current: initial,
            scenes,
            first_init: false,
        }
    }

    fn current(&mut self) -> &mut Box<dyn Stage + Send> {
        self.scenes.get_mut(self.current).unwrap()
    }

    pub async fn go(mut self, p: &mut Player, to: &'static str) -> Self {
        self.current = to;
        self.current().init(p).await;
        self
    }

    pub async fn update(mut self, p: &mut Player, dt: f32) -> Self {
        if !self.first_init {
            self.current().init(p).await;
            self.first_init = true;
        }
        self.current().update(p, dt).await;
        self
    }

    pub async fn event(mut self, p: &mut Player, ev: Event) -> Self {
        self.current().event(p, ev).await;
        self
    }

    pub async fn key(mut self, p: &mut Player, state: KeyState, key: Key) -> Self {
        self.current().key(p, state, key).await;
        self
    }

    pub fn view(mut self, frame: &mut Frame, depth: &wgpu::RawTextureView, target: &wgpu::RawTextureView) -> Self {
        self.current().view(frame, depth, target);
        self
    }
}
