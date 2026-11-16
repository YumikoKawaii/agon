use crate::context::Context;

mod battle;
pub mod characters;
mod lobby;
mod login;
mod main_menu;
mod rooms;

pub use self::{battle::Battle, characters::Characters, lobby::Lobby, login::Login, main_menu::MainMenu, rooms::Rooms};

pub enum StackCommand {
    None,
    Push(Box<dyn Screen>),
    Pop,
    Reset(Box<dyn Screen>),
    Quit,
}

pub trait Screen {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> StackCommand;
    fn draw(&self, ctx: &Context);
    fn resume(&mut self, _ctx: &mut Context) {}
    fn shows_cursor(&self) -> bool {
        true
    }
}

pub struct ScreenStack {
    screens: Vec<Box<dyn Screen>>,
    pub quit: bool,
}

impl ScreenStack {
    pub fn new(start: Box<dyn Screen>) -> Self {
        ScreenStack { screens: vec![start], quit: false }
    }

    fn top(&mut self) -> &mut Box<dyn Screen> {
        self.screens.last_mut().expect("screen stack is empty")
    }

    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        let command = self.top().update(ctx, dt);
        match command {
            StackCommand::None => {}
            StackCommand::Push(s) => self.screens.push(s),
            StackCommand::Pop if self.screens.len() > 1 => {
                self.screens.pop();
                self.top().resume(ctx);
            }
            StackCommand::Pop | StackCommand::Quit => self.quit = true,
            StackCommand::Reset(s) => self.screens = vec![s],
        }
    }

    pub fn draw(&self, ctx: &Context) {
        let top = self.screens.last().expect("screen stack is empty");
        macroquad::input::show_mouse(top.shows_cursor());
        top.draw(ctx);
    }
}

pub fn signed_out(ctx: &mut Context) -> StackCommand {
    ctx.sign_out();
    StackCommand::Reset(Box::new(Login::new()))
}
