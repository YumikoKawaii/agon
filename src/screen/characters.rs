use macroquad::prelude::*;
use crate::ui::{draw_text, measure_text};

use crate::{
    assets,
    context::Context,
    core::{anim::State, KITS},
    net::{api::UserReply, Pending},
    screen::{signed_out, Screen, StackCommand},
    ui::{self, Status},
};

const SOUTH: usize = 2;

pub fn draw_standing(kit: usize, feet: Vec2, scale: f32, flip: bool, tint: Color) {
    let tex = assets::skin(kit).frame(State::Idle, SOUTH, (get_time() * 60.0) as u32, 1);
    let hd = if tex.width() >= assets::HD_CELL { 0.5 } else { 1.0 };
    let size = vec2(tex.width(), tex.height()) * scale * hd;
    let bob = ((get_time() * 2.6).sin() as f32 * 0.5 + 0.5) * scale * 1.5;
    draw_texture_ex(tex, (feet.x - size.x / 2.0).round(), (feet.y - size.y * 0.86 - bob).round(), tint, DrawTextureParams { dest_size: Some(size), flip_x: flip, ..Default::default() });
}

struct Layout {
    x: f32,
    y: f32,
    hero: Vec2,
    info: Rect,
    tiles: Vec<Rect>,
    confirm: Rect,
    back: Rect,
}

const W: f32 = 1200.0;
const TILE: f32 = 92.0;
const ROW_TEXT_W: f32 = 540.0;

fn layout() -> Layout {
    let (x, y) = (ui::content_x(W), ui::oy());
    let tiles = (0..KITS.len()).map(|i| Rect::new(x + 20.0 + i as f32 * (TILE + 18.0), y + 540.0, TILE, TILE)).collect();
    Layout {
        x,
        y,
        hero: vec2(x + 230.0, y + 400.0),
        info: Rect::new(x + 480.0, y + 90.0, 700.0, 400.0),
        tiles,
        confirm: Rect::new(x + W - 250.0, y + 560.0, 230.0, 62.0),
        back: Rect::new(x + W - 400.0, y + 568.0, 130.0, 46.0),
    }
}

struct Row {
    icon: Option<&'static Texture2D>,
    key: &'static str,
    title: String,
    lines: Vec<String>,
}

fn rows(kit: usize) -> (Option<Row>, Vec<Row>) {
    let k = &KITS[kit];
    let parts: Vec<&str> = k.hint.split("   ").collect();
    let keys = ["LMB", "Q", "E", "R"];
    let split = |s: &str| -> (String, Vec<String>) {
        let (main, alt) = s.split_once(" / ").map_or((s, None), |(a, b)| (a, Some(b)));
        let mut title = main.to_string();
        if let Some(c) = title.get_mut(..1) {
            c.make_ascii_uppercase();
        }
        let lines = alt.map(|a| ui::wrap(a, ROW_TEXT_W, 18)).unwrap_or_default();
        (title, lines)
    };
    let skills = (0..4)
        .map(|i| {
            let part = parts.get(i).copied().unwrap_or("");
            let body = part.strip_prefix(keys[i]).unwrap_or(part).trim();
            let (title, lines) = split(body);
            Row { icon: assets::skill_icon(k.skills[i]), key: keys[i], title, lines }
        })
        .collect();
    let passive = parts.iter().find(|p| p.starts_with("PASSIVE")).map(|p| {
        let body = p.trim_start_matches("PASSIVE").trim();
        let text = body.split_once(':').map_or(body, |(_, t)| t.trim());
        Row { icon: Some(assets::passive_icon(k.passive)), key: "", title: k.passive.name().to_string(), lines: ui::wrap(text, ROW_TEXT_W + 40.0, 18) }
    });
    (passive, skills)
}

fn row_h(r: &Row) -> f32 {
    (30.0 + r.lines.len() as f32 * 21.0).max(58.0)
}

pub struct Characters {
    pick: usize,
    pending: Option<Pending<UserReply>>,
    status: Status,
}

impl Characters {
    pub fn new(current: usize) -> Self {
        Characters { pick: current, pending: None, status: Status::default() }
    }
}

impl Screen for Characters {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        ui::frame(20.0, 650.0);
        if let Some(p) = self.pending.as_mut() {
            if let Some(result) = p.poll() {
                self.pending = None;
                match result {
                    Ok(r) => {
                        ctx.user.kit = r.user.kit;
                        return StackCommand::Pop;
                    }
                    Err(e) if e.unauthenticated => return signed_out(ctx),
                    Err(e) => self.status.error = e.message,
                }
            }
        }
        let idle = self.pending.is_none();
        self.status.busy = !idle;
        let l = layout();
        if let Some(i) = l.tiles.iter().position(|r| ui::clicked(*r, idle)) {
            self.pick = i;
        }
        if idle && (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) {
            self.pick = (self.pick + 1) % KITS.len();
        }
        if idle && (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) {
            self.pick = (self.pick + KITS.len() - 1) % KITS.len();
        }
        if ui::clicked(l.confirm, idle) || (idle && is_key_pressed(KeyCode::Enter)) {
            self.pending = Some(ctx.api.set_kit(self.pick));
        }
        if ui::clicked(l.back, idle) || (idle && is_key_pressed(KeyCode::Escape)) {
            return StackCommand::Pop;
        }
        StackCommand::None
    }

    fn draw(&self, _ctx: &Context) {
        ui::frame(20.0, 650.0);
        let l = layout();
        let k = &KITS[self.pick];
        ui::text_out("Choose your character", l.x + 20.0, l.y + 56.0, ui::HEADING, ui::GOLD, 2.0);

        ui::pedestal(l.hero, 300.0);
        draw_standing(self.pick, l.hero, 4.6, false, WHITE);
        let id = Rect::new(l.hero.x - 170.0, l.hero.y + 34.0, 340.0, 50.0);
        ui::text_out_in(k.name, id, id.y + 40.0, 54, WHITE, 2.0);
        let role = k.role.name();
        let hp = format!("HP {}", k.hp);
        let cw = ui::chip_w(role, 20) + 10.0 + ui::chip_w(&hp, 20);
        let cx = l.hero.x - cw / 2.0;
        let r = ui::chip(role, cx, id.y + 60.0, 20, ui::INK, ui::CREAM);
        ui::chip(&hp, r.x + r.w + 10.0, id.y + 60.0, 20, ui::INK, ui::CREAM);

        let (passive, skills) = rows(self.pick);
        let mut h = 40.0 + skills.iter().map(row_h).sum::<f32>() + 8.0 * 3.0;
        if let Some(p) = &passive {
            h += row_h(p) + 22.0;
        }
        let info = Rect::new(l.info.x, l.info.y, l.info.w, h.max(l.info.h));
        ui::panel_scaled(info, 0.55);
        let mut y = info.y + 26.0;
        let x = info.x + 28.0;
        if let Some(p) = &passive {
            if let Some(t) = p.icon {
                ui::icon(t, Rect::new(x, y + 4.0, 46.0, 46.0));
            }
            draw_text(&p.title, x + 62.0, y + 22.0, 24.0, ui::INK);
            let tw = measure_text(&p.title, None, 24, 1.0).width;
            draw_text("passive", x + 70.0 + tw, y + 22.0, 18.0, ui::LAV_D);
            for (i, line) in p.lines.iter().enumerate() {
                draw_text(line, x + 62.0, y + 44.0 + i as f32 * 21.0, 18.0, Color::new(0.36, 0.32, 0.46, 1.0));
            }
            y += row_h(p) + 8.0;
            let mut dx = x;
            while dx < info.x + info.w - 40.0 {
                draw_rectangle(dx, y, 8.0, 2.0, ui::with_alpha(ui::INK, 0.25));
                dx += 14.0;
            }
            y += 14.0;
        }
        for row in &skills {
            let rh = row_h(row);
            let rr = Rect::new(x - 10.0, y - 4.0, info.w - 36.0, rh);
            if ui::hovered(rr) {
                draw_rectangle(rr.x, rr.y, rr.w, rr.h, WHITE);
                draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, 3.0, ui::GOLD);
            }
            if let Some(t) = row.icon {
                ui::icon(t, Rect::new(x, y + 2.0, 46.0, 46.0));
            }
            ui::keycap(row.key, x + 82.0, y + 14.0, 16);
            draw_text(&row.title, x + 112.0, y + 24.0, 22.0, ui::INK);
            for (i, line) in row.lines.iter().enumerate() {
                draw_text(line, x + 112.0, y + 46.0 + i as f32 * 21.0, 18.0, Color::new(0.36, 0.32, 0.46, 1.0));
            }
            y += rh + 8.0;
        }

        for (i, r) in l.tiles.iter().enumerate() {
            let on = i == self.pick;
            let hov = ui::hovered(*r);
            let lift = if hov && !on { 3.0 } else { 0.0 };
            let rr = Rect::new(r.x, r.y - lift, r.w, r.h);
            if on {
                draw_rectangle(rr.x - 7.0, rr.y - 7.0, rr.w + 14.0, rr.h + 14.0, ui::INK);
                draw_rectangle(rr.x - 5.0, rr.y - 5.0, rr.w + 10.0, rr.h + 10.0, ui::GOLD);
            }
            ui::portrait_box(i, rr, if on { ui::CREAM } else { Color::new(0.3, 0.27, 0.42, 1.0) }, false);
            let c = if on { ui::GOLD } else if hov { WHITE } else { ui::PALE };
            ui::text_out_in(KITS[i].name, Rect::new(r.x - 10.0, 0.0, r.w + 20.0, 0.0), r.y + r.h + 30.0, 20, c, 1.0);
        }
        ui::ghost(l.back, "Back", self.pending.is_none());
        ui::mint_button(l.confirm, "Confirm", self.pending.is_none());
        self.status.draw();
    }
}
