use macroquad::prelude::*;

mod assets;
mod context;
mod core;
mod error;
mod net;
mod screen;
mod ui;

struct MainState {
    ctx: context::Context,
    screens: screen::ScreenStack,
}

impl MainState {
    fn new() -> Self {
        MainState { ctx: context::Context::default(), screens: screen::ScreenStack::new(Box::new(screen::Login::new())) }
    }

    fn tick(&mut self) {
        self.screens.update(&mut self.ctx, get_frame_time().min(0.25));
        clear_background(ui::BG);
        self.screens.draw(&self.ctx);
    }
}

fn window_conf() -> Conf {
    Conf { window_title: "Agon".to_owned(), window_width: 1280, window_height: 720, ..Default::default() }
}

#[macroquad::main(window_conf)]
async fn main() {
    assets::load().await.expect("can't load assets");
    let mut state = MainState::new();
    while !state.screens.quit {
        state.tick();
        next_frame().await;
    }
}
