use macroquad::prelude::*;

use crate::{
    assets,
    context::Context,
    core::kit::kit,
    screen::{characters::draw_standing, Battle, Characters, Login, Rooms, Screen, StackCommand},
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

const W: f32 = 1200.0;
const MODES: [(Item, &str, &str); 3] = [(Item::Practice, "Vs Bot", "pick a rival"), (Item::Training, "Training", "dummy target"), (Item::Watch, "Bot vs Bot", "watch a match")];

struct Layout {
    x: f32,
    y: f32,
    hero: Vec2,
    hero_hit: Rect,
    change: Rect,
    play: Rect,
    modes: [Rect; 3],
    sign_out: Rect,
    quit: Rect,
}

fn layout() -> Layout {
    let (x, y) = (ui::content_x(W), ui::oy());
    let mx = x + 560.0;
    let mw = (W - 560.0 - 20.0 - 2.0 * 22.0) / 3.0;
    Layout {
        x,
        y,
        hero: vec2(x + 250.0, y + 430.0),
        hero_hit: Rect::new(x + 110.0, y + 140.0, 280.0, 320.0),
        change: Rect::new(x + 160.0, y + 560.0, 180.0, 40.0),
        play: Rect::new(mx, y + 150.0, W - 580.0, 160.0),
        modes: [0, 1, 2].map(|i| Rect::new(mx + i as f32 * (mw + 22.0), y + 350.0, mw, 170.0)),
        sign_out: Rect::new(x + W - 250.0, y + 580.0, 120.0, 40.0),
        quit: Rect::new(x + W - 120.0, y + 580.0, 100.0, 40.0),
    }
}

pub struct MainMenu;

impl MainMenu {
    pub fn new() -> Self {
        MainMenu
    }
}

impl Screen for MainMenu {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        ui::frame(20.0, 640.0);
        let l = layout();
        let mut items = vec![(Item::Online, l.play), (Item::Characters, l.change), (Item::Characters, l.hero_hit), (Item::SignOut, l.sign_out), (Item::Quit, l.quit)];
        items.extend(MODES.iter().zip(l.modes).map(|(m, r)| (m.0, r)));
        let item = if is_key_pressed(KeyCode::Enter) { Some(Item::Online) } else { items.into_iter().find(|(_, r)| ui::clicked(*r, true)).map(|(i, _)| i) };
        let Some(item) = item else {
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
        ui::frame(20.0, 640.0);
        let l = layout();
        let logo = &assets::get().ui.logo;
        let ls = 1.3;
        draw_texture_ex(logo, l.x + 10.0, l.y + 10.0, WHITE, DrawTextureParams { dest_size: Some(vec2(logo.width(), logo.height()) * ls), ..Default::default() });

        let name = ctx.user.username.clone();
        let nw = ui::chip_w(&name, 20);
        ui::chip(&name, l.x + W - 20.0 - nw, l.y + 40.0, 20, ui::INK, ui::CREAM);

        let k = kit(ctx.user.kit);
        ui::pedestal(l.hero, 320.0);
        let hov = ui::hovered(l.hero_hit);
        draw_standing(ctx.user.kit, l.hero, 4.8, false, if hov { Color::new(1.08, 1.08, 1.08, 1.0) } else { WHITE });
        let id = Rect::new(l.hero.x - 170.0, l.hero.y + 30.0, 340.0, 50.0);
        ui::text_out_in(k.name, id, id.y + 42.0, 52, WHITE, 2.0);
        ui::text_in(&format!("{}  ·  HP {}", k.role.name(), k.hp), id, id.y + 76.0, ui::SMALL, ui::PALE);
        ui::ghost(l.change, "Change character", true);

        let p = l.play;
        let h = ui::hovered(p);
        let y = if h && is_mouse_button_down(MouseButton::Left) { 3.0 } else { 0.0 };
        assets::get().ui.button.draw(Rect::new(p.x, p.y + y, p.w, p.h), 3.2, if h { Color::new(1.1, 1.1, 1.1, 1.0) } else { WHITE });
        ui::text_out_in("PLAY ONLINE", p, p.y + y + 70.0, 64, WHITE, 2.5);
        ui::text_in("create a room or join by id", p, p.y + y + 100.0, ui::SMALL, ui::PALE);

        for (((_, label, sub), r), t) in MODES.iter().zip(l.modes).zip(&assets::get().ui.modes) {
            ui::card(r, false);
            let lift = if ui::hovered(r) { 3.0 } else { 0.0 };
            let s = 80.0;
            draw_texture_ex(t, r.x + (r.w - s) / 2.0, r.y + 16.0 - lift, WHITE, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
            ui::text_in(label, r, r.y + 124.0 - lift, 28, ui::INK);
            ui::text_in(sub, r, r.y + 148.0 - lift, 18, Color::new(0.36, 0.32, 0.46, 1.0));
        }
        ui::ghost(l.sign_out, "Sign out", true);
        ui::ghost(l.quit, "Quit", true);
    }
}
