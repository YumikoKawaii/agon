use macroquad::prelude::*;

use crate::{
    context::Context,
    net::{api::Auth, Pending},
    screen::{MainMenu, Screen, StackCommand},
    ui::{self, Status, TextField},
};

const USER_MAX: usize = 16;
const PASS_MAX: usize = 64;

struct Layout {
    user: Rect,
    pass: Rect,
    sign_in: Rect,
    register: Rect,
}

fn layout() -> Layout {
    let actions = ui::row(ui::centered(360.0, 430.0, 52.0), 2, 12.0);
    Layout { user: ui::centered(360.0, 260.0, 48.0), pass: ui::centered(360.0, 350.0, 48.0), sign_in: actions[0], register: actions[1] }
}

pub struct Login {
    user: TextField,
    pass: TextField,
    focus: usize,
    pending: Option<Pending<Auth>>,
    status: Status,
}

impl Login {
    pub fn new() -> Self {
        Login { user: TextField::name(USER_MAX), pass: TextField::secret(PASS_MAX), focus: 0, pending: None, status: Status::default() }
    }
}

impl Screen for Login {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        let l = layout();
        if let Some(p) = self.pending.as_mut() {
            if let Some(result) = p.poll() {
                self.pending = None;
                match result {
                    Ok(auth) => {
                        ctx.sign_in(auth);
                        self.pass.value.clear();
                        return StackCommand::Push(Box::new(MainMenu::new()));
                    }
                    Err(e) => self.status.error = e.message,
                }
            }
        }
        self.status.busy = self.pending.is_some();
        if ui::clicked(l.user, true) {
            self.focus = 0;
        }
        if ui::clicked(l.pass, true) {
            self.focus = 1;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.focus = 1 - self.focus;
        }
        if self.focus == 0 { self.user.input() } else { self.pass.input() }
        let idle = self.pending.is_none();
        let sign_in = ui::clicked(l.sign_in, idle) || (idle && is_key_pressed(KeyCode::Enter));
        let register = ui::clicked(l.register, idle);
        if sign_in || register {
            self.status.error.clear();
            let (u, p) = (&self.user.value, &self.pass.value);
            self.pending = Some(if register { ctx.api.register(u, p) } else { ctx.api.login(u, p) });
        }
        StackCommand::None
    }

    fn draw(&self, _ctx: &Context) {
        let l = layout();
        let idle = self.pending.is_none();
        ui::logo(110.0, 1.6);
        ui::text_centered("sign in or create an account", 190.0, ui::SMALL + 2, ui::DIM);
        ui::form(205.0, 310.0);
        self.user.draw(l.user, "username", self.focus == 0);
        self.pass.draw(l.pass, "password", self.focus == 1);
        ui::button(l.sign_in, "Sign in", idle);
        ui::button(l.register, "Register", idle);
        self.status.draw();
    }
}
