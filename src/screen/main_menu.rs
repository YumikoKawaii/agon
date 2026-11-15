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
    Characters,
    SignOut,
    Quit,
}

const ITEMS: [(Item, &str); 5] = [
    (Item::Online, "Play online"),
    (Item::Practice, "Practice"),
    (Item::Characters, "Characters"),
    (Item::SignOut, "Sign out"),
    (Item::Quit, "Quit"),
];

fn layout() -> Vec<(Item, &'static str, Rect)> {
    ITEMS.iter().enumerate().map(|(i, (item, label))| (*item, *label, ui::centered(320.0, 250.0 + i as f32 * 66.0, 56.0))).collect()
}

pub struct MainMenu;

impl MainMenu {
    pub fn new() -> Self {
        MainMenu
    }
}

impl Screen for MainMenu {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        let Some(item) = layout().into_iter().find(|(_, _, r)| ui::clicked(*r, true)).map(|(i, _, _)| i) else {
            return StackCommand::None;
        };
        match item {
            Item::Online => StackCommand::Push(Box::new(Rooms::new())),
            Item::Practice => StackCommand::Push(Box::new(Battle::practice(ctx.user.kit))),
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
        ui::text_centered("AGON", 160.0, ui::TITLE, ui::ACCENT);
        ui::text_centered(&format!("signed in as {}   |   {}", ctx.user.username, kit(ctx.user.kit).name), 205.0, ui::SMALL + 2, ui::DIM);
        for (_, label, r) in layout() {
            ui::button(r, label, true);
        }
    }
}
