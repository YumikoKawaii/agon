use macroquad::prelude::*;

use crate::{
    context::Context,
    core::kit::kit,
    screen::{Battle, Characters, Login, Rooms, Screen, StackCommand},
    ui,
};

#[derive(Clone, Copy)]
enum Item {
    Online,
    Practice,
    Training,
    Watch,
    Characters,
    SignOut,
    Quit,
}

const ITEMS: [(Item, &str); 7] = [
    (Item::Online, "Play online"),
    (Item::Practice, "Vs Bot"),
    (Item::Training, "Training"),
    (Item::Watch, "Bot vs Bot"),
    (Item::Characters, "Characters"),
    (Item::SignOut, "Sign out"),
    (Item::Quit, "Quit"),
];

fn layout() -> Vec<(Item, &'static str, Rect)> {
    let x = screen_width() / 2.0 - 360.0;
    ITEMS.iter().enumerate().map(|(i, (item, label))| (*item, *label, Rect::new(x, 230.0 + ui::oy() + i as f32 * 58.0, 320.0, 50.0))).collect()
}

fn showcase() -> Rect {
    Rect::new(screen_width() / 2.0 + 40.0, 230.0 + ui::oy(), 320.0, 386.0)
}

pub struct MainMenu;

impl MainMenu {
    pub fn new() -> Self {
        MainMenu
    }
}

impl Screen for MainMenu {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        ui::frame(60.0, 640.0);
        let Some(item) = layout().into_iter().find(|(_, _, r)| ui::clicked(*r, true)).map(|(i, _, _)| i) else {
            return StackCommand::None;
        };
        match item {
            Item::Online => StackCommand::Push(Box::new(Rooms::new())),
            Item::Practice => StackCommand::Push(Box::new(Battle::practice(ctx.user.kit))),
            Item::Training => StackCommand::Push(Box::new(Battle::training(ctx.user.kit))),
            Item::Watch => StackCommand::Push(Box::new(Battle::watch(ctx.user.kit))),
            Item::Characters => StackCommand::Push(Box::new(Characters::new(ctx.user.kit))),
            Item::SignOut => {
                drop(ctx.api.logout());
                ctx.sign_out();
                StackCommand::Reset(Box::new(Login::new()))
            }
            Item::Quit => StackCommand::Quit,
        }
    }

    fn draw(&self, ctx: &Context) {
        ui::frame(60.0, 640.0);
        ui::logo(110.0, 1.6);
        let s = showcase();
        ui::panel(s, false);
        crate::screen::characters::draw_idle(ctx.user.kit, s.x, s.y + 30.0, s.w, 3.5);
        ui::text_in(kit(ctx.user.kit).name, s, s.y + 320.0, 30, ui::INK);
        ui::text_in(&ctx.user.username, s, s.y + 352.0, ui::SMALL, ui::INK);
        for (_, label, r) in layout() {
            ui::button(r, label, true);
        }
    }
}
