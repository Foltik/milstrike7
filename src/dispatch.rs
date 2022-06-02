use lib::prelude::*;
use async_trait::async_trait;

use crate::Model;
use crate::demo::Event;

#[async_trait]
pub trait Render {
    async fn init(&mut self, app: &App, m: &mut Model);
    async fn update(&mut self, app: &App, m: &mut Model, dt: f32);
    async fn midi(&mut self, app: &App, m: &mut Model, ev: Event);

    fn view(&mut self, frame: &mut Frame, view: &wgpu::RawTextureView);
}

pub struct Dispatch {
    i: usize,
    init: bool,
    scenes: Vec<Box<dyn Render + Send>>,
}

impl Dispatch {
    pub fn new(device: &wgpu::Device, scenes: Vec<Box<dyn Render + Send>>) -> Self {
        Self {
            i: 0,
            init: false,
            scenes
        }
    }


    fn current(&mut self) -> &mut Box<dyn Render + Send> {
        &mut self.scenes[self.i]
    }

    pub async fn go(mut self, app: &App, m: &mut Model, i: i32) -> Self {
        self.i = i.rem_euclid(self.scenes.len() as i32) as usize;
        self.current().init(app, m).await;
        self
    }

    pub async fn go_next(self, app: &App, m: &mut Model) -> Self {
        let i = self.i as i32 + 1;
        self.go(app, m, i).await
    }

    pub async fn go_prev(self, app: &App, m: &mut Model) -> Self {
        let i = self.i as i32 - 1;
        self.go(app, m, i).await
    }

    pub async fn update(mut self, app: &App, m: &mut Model, dt: f32) -> Self {
        if !self.init {
            self.current().init(app, m).await;
            self.init = true;
        }
        self.current().update(app, m, dt).await;
        self
    }

    pub async fn midi(mut self, app: &App, m: &mut Model, ev: Event) -> Self {
        self.current().midi(app, m, ev).await;
        self
    }

    pub fn encode(mut self, frame: &mut Frame, view: &wgpu::RawTextureView) -> Self {
        self.current().view(frame, view);
        self
    }
}
