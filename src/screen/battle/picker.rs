use macroquad::prelude::*;

use crate::{
    assets,
    core::{
        bot,
        kit::{Role, KITS},
    },
    ui,
};

use super::view::VIEW;

const COLS: usize = 5;
const ROWS: usize = 2;
const TILE: f32 = 76.0;
const CELL: Vec2 = vec2(92.0, 108.0);
const RECENT_MAX: usize = 5;

pub enum Action {
    Pick(usize, usize),
    Level(usize),
    Swap,
    Reset,
}

#[derive(Default)]
pub struct Picker {
    pub open: Option<usize>,
    chosen: Option<usize>,
    scroll: usize,
    role: Option<Role>,
    search: String,
    recent: Vec<usize>,
    typing: bool,
}

const SEARCH_MAX: usize = 16;

pub struct Bar {
    pub level: Option<usize>,
    pub kits: [usize; 2],
}

fn bar_rect() -> Rect {
    Rect::new(VIEW.x / 2.0 - 250.0, 4.0, 500.0, 98.0)
}

fn chip_rect(side: usize) -> Rect {
    let b = bar_rect();
    Rect::new(b.x + 24.0 + side as f32 * 104.0, b.y + 36.0, 50.0, 50.0)
}

fn level_rect(i: usize) -> Rect {
    let b = bar_rect();
    Rect::new(b.x + 188.0 + i as f32 * 64.0, b.y + 44.0, 62.0, 36.0)
}

fn swap_rect() -> Rect {
    let b = bar_rect();
    Rect::new(b.x + 390.0, b.y + 22.0, 88.0, 32.0)
}

fn reset_rect() -> Rect {
    let b = bar_rect();
    Rect::new(b.x + 390.0, b.y + 56.0, 88.0, 32.0)
}

fn panel_rect() -> Rect {
    Rect::new(VIEW.x / 2.0 - 380.0, 108.0, 760.0, 430.0)
}

fn search_rect() -> Rect {
    let p = panel_rect();
    Rect::new(p.x + p.w - 270.0, p.y + 22.0, 240.0, 36.0)
}

fn tab_rect(i: usize) -> Rect {
    let p = panel_rect();
    Rect::new(p.x + 28.0 + i as f32 * 96.0, p.y + 70.0, 90.0, 30.0)
}

fn grid_origin() -> Vec2 {
    let p = panel_rect();
    vec2(p.x + 28.0, p.y + 114.0)
}

fn scrollbar_rect() -> Rect {
    let o = grid_origin();
    Rect::new(o.x + CELL.x * COLS as f32 + 4.0, o.y, 10.0, CELL.y * ROWS as f32)
}

fn preview_rect() -> Rect {
    let p = panel_rect();
    Rect::new(p.x + 524.0, p.y + 70.0, 210.0, 300.0)
}

fn pick_rect() -> Rect {
    let r = preview_rect();
    Rect::new(r.x + 20.0, r.y + r.h - 44.0, r.w - 40.0, 42.0)
}

fn recent_rect(i: usize) -> Rect {
    let p = panel_rect();
    Rect::new(p.x + 120.0 + i as f32 * 50.0, p.y + p.h - 64.0, 42.0, 42.0)
}

fn skill_rect(k: usize) -> Rect {
    let r = preview_rect();
    Rect::new(r.x + 13.0 + k as f32 * 48.0, r.y + 192.0, 40.0, 40.0)
}

fn text_mid(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = measure_text(t, None, size as u16, 1.0).width;
    draw_text(t, cx - w / 2.0, y, size, c);
}

fn button(r: Rect, label: &str, on: bool, hover: bool) {
    let tint = if on { Color::new(1.0, 0.92, 0.6, 1.0) } else if hover { Color::new(1.1, 1.1, 1.1, 1.0) } else { WHITE };
    ui::slot(r, tint);
    let size = if r.h < 40.0 { 17.0 } else { 20.0 };
    text_mid(label, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.3, size, ui::INK);
}

fn chip(r: Rect, kit: Option<usize>, on: bool, hover: bool) {
    draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color::new(0.9, 0.96, 0.92, 1.0) } else { Color::new(0.82, 0.9, 0.86, 1.0) });
    match kit {
        Some(k) => draw_texture_ex(assets::portrait(k), r.x + 3.0, r.y + 3.0, WHITE, DrawTextureParams { dest_size: Some(r.size() - vec2(6.0, 6.0)), ..Default::default() }),
        None => text_mid("?", r.x + r.w / 2.0, r.y + r.h / 2.0 + 12.0, r.h * 0.6, ui::INK),
    }
    ui::frame_border(r);
    if on {
        draw_rectangle_lines(r.x - 4.0, r.y - 4.0, r.w + 8.0, r.h + 8.0, 4.0, ui::GOLD);
    }
}

impl Picker {
    fn entries(&self) -> Vec<Option<usize>> {
        let q = self.search.to_lowercase();
        let mut v: Vec<Option<usize>> = (0..KITS.len()).filter(|i| self.role.is_none_or(|r| KITS[*i].role == r) && KITS[*i].name.to_lowercase().contains(&q)).map(Some).collect();
        if self.role.is_none() && q.is_empty() {
            v.push(None);
        }
        v
    }

    fn max_scroll(&self) -> usize {
        self.entries().len().div_ceil(COLS).saturating_sub(ROWS)
    }

    fn tile_at(&self, m: Vec2) -> Option<(usize, Option<usize>)> {
        let o = grid_origin();
        let entries = self.entries();
        for (n, e) in entries.iter().enumerate().skip(self.scroll * COLS).take(COLS * ROWS) {
            let i = n - self.scroll * COLS;
            let r = Rect::new(o.x + (i % COLS) as f32 * CELL.x + (CELL.x - TILE) / 2.0, o.y + (i / COLS) as f32 * CELL.y, TILE, TILE);
            if r.contains(m) {
                return Some((n, *e));
            }
        }
        None
    }

    pub fn blocks(&self, m: Vec2) -> bool {
        self.open.is_some() || bar_rect().contains(m)
    }

    fn open_for(&mut self, side: usize, current: usize) {
        if self.open == Some(side) {
            self.open = None;
            return;
        }
        self.open = Some(side);
        self.chosen = Some(current);
        self.search.clear();
        self.typing = false;
        self.role = None;
        self.scroll = 0;
    }

    fn commit(&mut self, side: usize, kit: Option<usize>) -> Action {
        let kit = kit.unwrap_or_else(|| macroquad::rand::gen_range(0, KITS.len()));
        self.recent.retain(|k| *k != kit);
        self.recent.insert(0, kit);
        self.recent.truncate(RECENT_MAX);
        self.open = None;
        Action::Pick(side, kit)
    }

    pub fn update(&mut self, m: Vec2, bar: &Bar) -> Option<Action> {
        let typing = self.open.is_some() && self.typing;
        while let Some(c) = get_char_pressed() {
            if typing && (c.is_ascii_alphanumeric() || c == ' ') && self.search.len() < SEARCH_MAX {
                self.search.push(c);
                self.scroll = 0;
            }
        }
        if typing && is_key_pressed(KeyCode::Backspace) {
            self.search.pop();
        }
        if self.open.is_none() && is_key_pressed(KeyCode::C) {
            self.open_for(0, bar.kits[0]);
            return None;
        }
        if let Some(side) = self.open {
            if is_key_pressed(KeyCode::Tab) {
                let other = 1 - side;
                self.open_for(other, bar.kits[other]);
                return None;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                self.typing = search_rect().contains(m);
            }
            let (_, wheel) = mouse_wheel();
            if wheel != 0.0 && Rect::new(grid_origin().x, grid_origin().y, CELL.x * COLS as f32 + 20.0, CELL.y * ROWS as f32).contains(m) {
                self.scroll = if wheel < 0.0 { (self.scroll + 1).min(self.max_scroll()) } else { self.scroll.saturating_sub(1) };
            }
            if !is_mouse_button_pressed(MouseButton::Left) {
                return None;
            }
            if let Some((_, kit)) = self.tile_at(m) {
                if self.chosen == kit {
                    return Some(self.commit(side, kit));
                }
                self.chosen = kit;
                return None;
            }
            if pick_rect().contains(m) {
                let kit = self.chosen;
                return Some(self.commit(side, kit));
            }
            for (i, k) in self.recent.clone().into_iter().enumerate() {
                if recent_rect(i).contains(m) {
                    return Some(self.commit(side, Some(k)));
                }
            }
            for i in 0..=Role::ALL.len() {
                if tab_rect(i).contains(m) {
                    self.role = i.checked_sub(1).map(|r| Role::ALL[r]);
                    self.scroll = 0;
                    return None;
                }
            }
            if panel_rect().contains(m) {
                return None;
            }
        }
        if !is_mouse_button_pressed(MouseButton::Left) || !bar_rect().contains(m) {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.open = None;
            }
            return None;
        }
        for side in 0..2 {
            if chip_rect(side).contains(m) {
                self.open_for(side, bar.kits[side]);
                return None;
            }
        }
        if bar.level.is_some() {
            for i in 0..bot::LEVELS.len() {
                if level_rect(i).contains(m) {
                    return Some(Action::Level(i));
                }
            }
        }
        if swap_rect().contains(m) {
            self.open = None;
            return Some(Action::Swap);
        }
        if reset_rect().contains(m) {
            self.open = None;
            return Some(Action::Reset);
        }
        None
    }

    pub fn draw_bar(&self, m: Vec2, bar: &Bar) {
        let b = bar_rect();
        ui::panel_scaled(b, 0.45);
        let foe = if bar.level.is_some() { "BOT" } else { "DUMMY" };
        for (side, label) in [(0, "YOU"), (1, foe)] {
            let r = chip_rect(side);
            text_mid(label, r.x + r.w / 2.0, r.y - 6.0, 16.0, ui::INK);
            chip(r, Some(bar.kits[side]), self.open == Some(side), r.contains(m));
            if side == 0 {
                draw_rectangle(r.x - 6.0, r.y + r.h - 14.0, 18.0, 18.0, ui::INK);
                text_mid("C", r.x + 3.0, r.y + r.h, 16.0, WHITE);
            }
            draw_triangle(vec2(r.x + r.w - 12.0, r.y + r.h - 9.0), vec2(r.x + r.w - 4.0, r.y + r.h - 9.0), vec2(r.x + r.w - 8.0, r.y + r.h - 4.0), ui::INK);
        }
        let l = chip_rect(0);
        text_mid("VS", l.x + l.w + 27.0, l.y + 34.0, 26.0, ui::INK);
        match bar.level {
            Some(level) => {
                text_mid("BOT LEVEL", level_rect(1).x + 31.0, level_rect(1).y - 6.0, 16.0, ui::INK);
                for (i, (name, _)) in bot::LEVELS.iter().enumerate() {
                    let r = level_rect(i);
                    let mut label = name.to_string();
                    label[..1].make_ascii_uppercase();
                    button(r, &label, i == level, r.contains(m));
                }
            }
            None => text_mid("TRAINING", level_rect(1).x + 31.0, level_rect(1).y + 24.0, 22.0, ui::INK),
        }
        button(swap_rect(), "Swap", false, swap_rect().contains(m));
        button(reset_rect(), "Reset", false, reset_rect().contains(m));
    }

    pub fn draw_panel(&self, m: Vec2) {
        let Some(side) = self.open else { return };
        let p = panel_rect();
        ui::panel_scaled(p, 0.7);
        draw_text(if side == 0 { "Choose your character" } else { "Choose opponent" }, p.x + 30.0, p.y + 50.0, 28.0, ui::INK);
        draw_text("Tab: switch side   Esc: close", p.x + 400.0, p.y + p.h - 36.0, 16.0, ui::INK);
        let s = search_rect();
        draw_rectangle(s.x, s.y, s.w, s.h, Color::new(1.0, 0.98, 0.94, 1.0));
        draw_rectangle_lines(s.x, s.y, s.w, s.h, if self.typing { 3.0 } else { 2.0 }, if self.typing { ui::GOLD } else { ui::INK });
        let caret = if self.typing && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
        let (txt, col) = if self.search.is_empty() && !self.typing { ("click to search".to_string(), Color::new(0.55, 0.52, 0.62, 1.0)) } else { (format!("{}{caret}", self.search), ui::INK) };
        draw_text(&txt, s.x + 10.0, s.y + 25.0, 18.0, col);
        for i in 0..=Role::ALL.len() {
            let r = tab_rect(i);
            let label = i.checked_sub(1).map_or("All", |k| Role::ALL[k].name());
            let on = self.role == i.checked_sub(1).map(|k| Role::ALL[k]);
            button(r, label, on, r.contains(m));
        }
        let o = grid_origin();
        let hovered = self.tile_at(m);
        let entries = self.entries();
        for (n, e) in entries.iter().enumerate().skip(self.scroll * COLS).take(COLS * ROWS) {
            let i = n - self.scroll * COLS;
            let r = Rect::new(o.x + (i % COLS) as f32 * CELL.x + (CELL.x - TILE) / 2.0, o.y + (i / COLS) as f32 * CELL.y, TILE, TILE);
            chip(r, *e, self.chosen == *e, hovered.is_some_and(|(h, _)| h == n));
            let name = e.map_or("Random", |k| KITS[k].name);
            let short = if name.len() > 11 { format!("{}.", &name[..10]) } else { name.to_string() };
            text_mid(&short, r.x + r.w / 2.0, r.y + r.h + 18.0, 15.0, ui::INK);
        }
        if entries.is_empty() {
            text_mid("no match", o.x + CELL.x * 2.5, o.y + CELL.y, 22.0, ui::INK);
        }
        let sb = scrollbar_rect();
        draw_rectangle(sb.x, sb.y, sb.w, sb.h, Color::new(0.88, 0.83, 0.72, 1.0));
        let rows = entries.len().div_ceil(COLS).max(ROWS);
        let th = sb.h * ROWS as f32 / rows as f32;
        draw_rectangle(sb.x, sb.y + sb.h * self.scroll as f32 / rows as f32, sb.w, th, Color::new(0.72, 0.62, 0.86, 1.0));
        draw_rectangle_lines(sb.x, sb.y, sb.w, sb.h, 1.5, ui::INK);
        let pr = preview_rect();
        draw_line(pr.x - 12.0, pr.y, pr.x - 12.0, pr.y + pr.h, 1.5, Color::new(0.72, 0.66, 0.54, 1.0));
        let shown = hovered.map(|(_, k)| k).unwrap_or(self.chosen);
        let big = Rect::new(pr.x + (pr.w - 112.0) / 2.0, pr.y, 112.0, 112.0);
        chip(big, shown, false, false);
        match shown {
            Some(k) => {
                let kit = &KITS[k];
                text_mid(kit.name, pr.x + pr.w / 2.0, pr.y + 140.0, 22.0, ui::INK);
                text_mid(&format!("{}   HP {}", kit.role.name(), kit.hp), pr.x + pr.w / 2.0, pr.y + 166.0, 16.0, ui::INK);
                let parts: Vec<&str> = kit.hint.split("   ").collect();
                let mut line = "hover a skill";
                for (i, s) in kit.skills.iter().enumerate() {
                    let r = skill_rect(i);
                    ui::slot(r, WHITE);
                    if let Some(t) = assets::skill_icon(*s) {
                        draw_texture_ex(t, r.x + 4.0, r.y + 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(r.w - 8.0, r.h - 8.0)), ..Default::default() });
                    }
                    if r.contains(m) {
                        line = parts.get(i).copied().unwrap_or("");
                    }
                }
                let short = if line.len() > 26 { format!("{}...", &line[..25]) } else { line.to_string() };
                text_mid(&short, pr.x + pr.w / 2.0, pr.y + 248.0, 14.0, ui::INK);
            }
            None => text_mid("Random", pr.x + pr.w / 2.0, pr.y + 140.0, 22.0, ui::INK),
        }
        button(pick_rect(), "Pick", true, pick_rect().contains(m));
        draw_text("Recent", p.x + 30.0, p.y + p.h - 36.0, 18.0, ui::INK);
        for (i, k) in self.recent.iter().enumerate() {
            let r = recent_rect(i);
            chip(r, Some(*k), false, r.contains(m));
        }
    }
}
