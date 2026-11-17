use std::f32::consts::{PI, TAU};

use macroquad::{prelude::*, rand::gen_range};
use crate::ui::raw::{draw_text, measure_text};

use crate::{
    assets,
    core::{
        anim::State,
        effect::{EffectKind, KINDS},
        Player, ARENA, RADIUS,
    },
};

pub const S: f32 = 3.0;
pub const SPRITE: f32 = 0.5;
pub const FOOT: f32 = 9.0;
pub const CHEST: f32 = 9.0;
const POST_H: f32 = 14.0;
const TILE: f32 = 20.0;
const HIT_FLASH_TICKS: u32 = 4;
pub const VIEW: Vec2 = vec2(1500.0, 900.0);
const ORIGIN: Vec2 = vec2(30.0, 120.0);

pub const ICE: Color = Color::new(0.55, 0.85, 1.0, 1.0);
pub const PALE: Color = Color::new(0.88, 0.97, 1.0, 1.0);
pub const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);
pub const AQUA: Color = Color::new(0.45, 1.0, 0.92, 1.0);
pub const LILAC: Color = Color::new(0.78, 0.55, 1.0, 1.0);
pub const MINT: Color = Color::new(0.72, 0.96, 0.78, 1.0);
pub const BLOOD: Color = Color::new(1.0, 0.56, 0.66, 1.0);
pub const SAKURA: Color = Color::new(1.0, 0.68, 0.84, 1.0);
const CAGE: Color = Color::new(0.72, 0.62, 0.86, 1.0);
const HIT_TINT: Color = Color::new(1.0, 0.6, 0.6, 1.0);

pub struct Letterbox {
    pub scale: f32,
    pub offset: Vec2,
}

pub fn letterbox() -> Letterbox {
    let screen = vec2(screen_width(), screen_height());
    let fit = (screen.x / VIEW.x).min(screen.y / VIEW.y);
    let scale = if fit >= 1.0 { fit.floor() } else { fit };
    Letterbox { scale, offset: ((screen - VIEW * scale) / 2.0).round() }
}

#[derive(Default)]
pub struct Camera {
    jitter: Vec2,
}

impl Camera {
    pub fn shake(&mut self, amount: f32) {
        self.jitter = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)) * amount;
    }

    pub fn proj(&self, v: Vec2) -> Vec2 {
        ORIGIN + v * S + self.jitter
    }

    pub fn air(&self, v: Vec2, z: f32) -> Vec2 {
        self.proj(v) - vec2(0.0, z * S)
    }

    pub fn mouse_view(&self) -> Vec2 {
        let l = letterbox();
        (Vec2::from(mouse_position()) - l.offset) / l.scale
    }

    pub fn mouse_world(&self) -> Vec2 {
        (self.mouse_view() - ORIGIN + vec2(0.0, FOOT * S)) / S
    }
}

pub fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a.clamp(0.0, 1.0))
}

pub fn glow(c: Vec2, r: f32, col: Color, a: f32) {
    draw_circle(c.x, c.y, r * 2.4, alpha(col, a * 0.12));
    draw_circle(c.x, c.y, r * 1.5, alpha(col, a * 0.3));
    draw_circle(c.x, c.y, r, alpha(col, a));
    draw_circle(c.x, c.y, r * 0.45, alpha(WHITE, a));
}

pub fn magic_circle(c: Vec2, r: f32, rot: f32, col: Color, a: f32, w: f32) {
    draw_circle_lines(c.x, c.y, r, w, alpha(col, a));
    draw_circle_lines(c.x, c.y, r * 0.82, w * 0.6, alpha(col, a * 0.8));
    draw_circle_lines(c.x, c.y, r * 0.4, w * 0.6, alpha(col, a * 0.7));
    for tri in 0..2 {
        let base = rot + tri as f32 * PI / 3.0;
        let v: Vec<Vec2> = (0..3).map(|i| c + Vec2::from_angle(base + i as f32 * TAU / 3.0) * r * 0.82).collect();
        draw_triangle_lines(v[0], v[1], v[2], w * 0.7, alpha(col, a * 0.9));
    }
    for i in 0..24 {
        let d = Vec2::from_angle(-rot * 1.5 + i as f32 * TAU / 24.0);
        let len = if i % 3 == 0 { 0.1 } else { 0.05 };
        let (p0, p1) = (c + d * r * 0.86, c + d * r * (0.86 + len));
        draw_line(p0.x, p0.y, p1.x, p1.y, w * 0.6, alpha(col, a));
    }
    for i in 0..6 {
        let p = c + Vec2::from_angle(rot + i as f32 * TAU / 6.0) * r * 0.61;
        draw_poly_lines(p.x, p.y, 4, r * 0.07, rot.to_degrees() * 2.0, w * 0.5, alpha(col, a));
    }
}

pub fn draw_sprite(tex: &Texture2D, at: Vec2, tint: Color) {
    let size = vec2(tex.width(), tex.height()) * SPRITE * S;
    draw_texture_ex(tex, (at.x - size.x / 2.0).round(), (at.y - size.y / 2.0 - FOOT * S).round(), tint, DrawTextureParams { dest_size: Some(size), ..Default::default() });
}

pub fn draw_floor(cam: &Camera) {
    let tiles = &assets::get().ui.tiles;
    let (nx, ny) = ((ARENA.x / TILE) as i32, (ARENA.y / TILE) as i32);
    for i in 0..nx {
        for j in 0..ny {
            let h = (i as u32).wrapping_mul(73_856_093) ^ (j as u32).wrapping_mul(19_349_663);
            let k = match h % 16 {
                0 => 12,
                1 | 2 => 1,
                3 => 4,
                _ => 0,
            };
            let p = cam.proj(vec2(i as f32 * TILE, j as f32 * TILE));
            draw_texture_ex(&tiles[k], p.x.round(), p.y.round(), WHITE, DrawTextureParams { dest_size: Some(vec2(TILE * S, TILE * S)), ..Default::default() });
        }
    }
}

pub fn draw_cage(cam: &Camera, front: bool) {
    if front {
        return;
    }
    let wall = &assets::get().ui.tiles[10];
    let (nx, ny) = ((ARENA.x / TILE) as i32, (ARENA.y / TILE) as i32);
    let mut cells = Vec::new();
    for i in -1..=nx {
        cells.push((i, -2));
        cells.push((i, -1));
        cells.push((i, ny));
    }
    for j in 0..ny {
        cells.push((-1, j));
        cells.push((nx, j));
    }
    for (i, j) in cells {
        let p = cam.proj(vec2(i as f32 * TILE, j as f32 * TILE));
        let tint = if j == -2 { Color::new(0.75, 0.75, 0.8, 1.0) } else { WHITE };
        draw_texture_ex(wall, p.x.round(), p.y.round(), tint, DrawTextureParams { dest_size: Some(vec2(TILE * S, TILE * S)), ..Default::default() });
    }
    let (a, b) = (cam.proj(Vec2::ZERO), cam.proj(ARENA));
    draw_rectangle_lines(a.x.round(), a.y.round(), (b.x - a.x).round(), (b.y - a.y).round(), 2.0, alpha(CAGE, 0.35));
}

pub fn draw_player(cam: &Camera, p: &Player, debug: bool) {
    let s = cam.proj(p.pos);
    draw_circle(s.x, s.y, RADIUS * S, Color::new(0.0, 0.0, 0.0, 0.45));
    if debug {
        if let Some(r) = p.hurtbox() {
            draw_circle(s.x, s.y, r * S, Color::new(0.35, 0.82, 1.0, 0.2));
            draw_circle_lines(s.x, s.y, r * S, 1.5, Color::from_rgba(90, 209, 255, 255));
        }
    }
    if p.untargetable {
        return;
    }
    draw_status(cam, p, false);
    use crate::core::Skill;
    let special = match p.last {
        Some(Skill::Overdrive) => Some(("transform", true)),
        Some(Skill::Starbreaker) => Some(("transform", false)),
        Some(Skill::AzureLance) => Some(("lance", true)),
        Some(Skill::GravityWell) => Some(("throw", true)),
        Some(Skill::RuneKnuckle) => Some(("punch", false)),
        Some(Skill::RocketDash) => Some(("dash", false)),
        _ => None,
    };
    let lit = !p.armed && p.fists > 0.0;
    let glow = energy_glow(p);
    let skin = |armed: bool| match (armed, glow) {
        (false, _) if lit => &assets::get().seren_lit,
        (true, Some(true)) => &assets::get().seren_charged,
        (false, Some(false)) => &assets::get().seren_fading,
        _ => assets::skin_in(p.kit, armed),
    };
    let tex = special
        .filter(|_| p.state == State::Attack)
        .and_then(|(clip, armed)| skin(armed).special(clip, p.dir8(), p.age, p.anim_ticks()).or_else(|| assets::skin_in(p.kit, armed).special(clip, p.dir8(), p.age, p.anim_ticks())))
        .unwrap_or_else(|| skin(p.armed).frame(p.state, p.dir8(), p.age, p.anim_ticks()));
    let tint = if p.state == State::Hit && p.age < HIT_FLASH_TICKS { HIT_TINT } else { status_tint(p) };
    draw_sprite(tex, s, tint);
    draw_status(cam, p, true);
    if debug {
        draw_text(&format!("{:?} {} d{}", p.state, p.frame, p.dir8()), s.x - 40.0, s.y - 22.0 * S, 20.0, WHITE);
    }
}

pub fn status_color(k: EffectKind) -> Color {
    match k {
        EffectKind::Slow => ICE,
        EffectKind::Root => LILAC,
        EffectKind::Bleed => BLOOD,
        EffectKind::Burn => MINT,
        EffectKind::Mark => GOLD,
        EffectKind::Shield => GOLD,
        EffectKind::Vulnerable => LILAC,
        EffectKind::Stun => PALE,
    }
}

pub fn status_name(k: EffectKind) -> &'static str {
    match k {
        EffectKind::Slow => "SLOW",
        EffectKind::Root => "ROOT",
        EffectKind::Bleed => "BLEED",
        EffectKind::Burn => "BURN",
        EffectKind::Mark => "MARK",
        EffectKind::Shield => "SHIELD",
        EffectKind::Vulnerable => "CURSE",
        EffectKind::Stun => "STUN",
    }
}

fn status_tint(p: &Player) -> Color {
    let pulse = 0.75 + 0.25 * (get_time() as f32 * 8.0).sin();
    let e = &p.effects;
    if e.stunned() {
        Color::new(0.8, 0.8, 0.85, 1.0)
    } else if e.on(EffectKind::Slow) {
        Color::new(0.7, 0.9, 1.0, 1.0)
    } else if e.on(EffectKind::Burn) {
        Color::new(0.85 + 0.15 * pulse, 0.8, 1.0, 1.0)
    } else if e.on(EffectKind::Vulnerable) {
        Color::new(0.9, 0.78, 1.0, 1.0)
    } else {
        WHITE
    }
}

fn status_z(k: EffectKind) -> f32 {
    match k {
        EffectKind::Stun => 24.0,
        EffectKind::Bleed => 16.0,
        _ => CHEST,
    }
}

fn draw_status(cam: &Camera, p: &Player, front: bool) {
    let all = &assets::get().status;
    let time = get_time() as f32;
    for k in KINDS {
        if !p.effects.on(k) {
            continue;
        }
        let st = &all[k.i()];
        if st.ground == front {
            continue;
        }
        let i = if k == EffectKind::Root {
            ((p.effects.progress(k) * st.sheet.frames as f32) as usize).min(st.sheet.frames - 1)
        } else {
            st.sheet.frame_at(time, true).unwrap_or(0)
        };
        let at = if st.ground { cam.proj(p.pos) } else { cam.air(p.pos, status_z(k)) };
        let fade = (p.effects.left(k) as f32 / 12.0).min(1.0);
        let a = if k == EffectKind::Shield { 0.8 * fade } else { fade };
        st.sheet.draw_at(i, at, st.anchor, SPRITE * S, a);
    }
}

const HP_FILL: [Color; 2] = [Color::new(0.5, 0.84, 0.76, 1.0), Color::new(0.91, 0.46, 0.54, 1.0)];
const SUBTLE: Color = Color::new(0.36, 0.32, 0.46, 1.0);
const DOCK: Rect = Rect { x: (VIEW.x - 440.0) / 2.0, y: VIEW.y - 126.0, w: 440.0, h: 118.0 };
const FOE: Rect = Rect { x: VIEW.x - 340.0, y: 8.0, w: 320.0, h: 84.0 };
const HALF: Vec2 = vec2(392.0, 118.0);
const TOP: Rect = Rect { x: VIEW.x / 2.0 - 260.0, y: 0.0, w: 520.0, h: 170.0 };

pub enum Hud {
    Focus { names: [String; 2], score: Option<[u32; 2]> },
    Equal { tags: [String; 2] },
}

fn half(i: usize) -> Rect {
    let x = if i == 0 { 14.0 } else { VIEW.x - 14.0 - HALF.x };
    Rect::new(x, 6.0, HALF.x, HALF.y)
}

pub fn hud_zones(hud: &Hud) -> [Rect; 3] {
    match hud {
        Hud::Focus { .. } => [Rect::new(DOCK.x, DOCK.y - 34.0, DOCK.w, DOCK.h + 34.0), FOE, TOP],
        Hud::Equal { .. } => {
            let grow = |r: Rect| Rect::new(r.x, r.y, r.w, r.h + 34.0);
            [grow(half(0)), grow(half(1)), TOP]
        }
    }
}

fn hp_frac(p: &Player) -> f32 {
    (p.hp.max(0) as f32 / p.max_hp() as f32).clamp(0.0, 1.0)
}

pub fn ease_lag(lag: f32, p: &Player, dt: f32) -> f32 {
    let f = hp_frac(p);
    if f >= lag {
        f
    } else {
        (lag - (lag - f).max(0.05) * dt * 2.2).max(f)
    }
}

fn hp_bar(r: Rect, p: &Player, lag: f32, side: usize, right: bool) {
    crate::ui::bar(r, hp_frac(p), lag, HP_FILL[side], right);
}

fn status_chips(p: &Player, x: f32, y: f32, right: bool) {
    let mut n = 0.0;
    for k in KINDS.iter().copied().filter(|k| p.effects.on(*k)) {
        let label = match k {
            EffectKind::Bleed => format!("{} x{}", status_name(k), p.effects.stacks),
            EffectKind::Shield => format!("{} {}", status_name(k), p.effects.power[k.i()]),
            _ => status_name(k).to_string(),
        };
        let tw = measure_text(&label, None, 16, 1.0).width + 16.0;
        let ex = if right { x - n - tw } else { x + n };
        let col = status_color(k);
        crate::ui::pill(Rect::new(ex, y, tw, 22.0), crate::ui::INK);
        draw_rectangle(ex + 8.0, y + 18.0, (tw - 16.0) * (1.0 - p.effects.progress(k)), 2.0, col);
        draw_text(&label, ex + 8.0, y + 16.0, 16.0, col);
        n += tw + 6.0;
    }
}

fn sweep(r: Rect, frac: f32, col: Color) {
    let frac = frac.clamp(0.0, 1.0);
    if frac <= 0.0 {
        return;
    }
    let c = r.center();
    let half = r.w / 2.0;
    let edge = |a: f32| {
        let d = Vec2::from_angle(a);
        c + d * (half / d.x.abs().max(d.y.abs()))
    };
    let steps = (64.0 * frac).ceil().max(1.0) as usize;
    let start = -PI / 2.0;
    let span = TAU * frac;
    for i in 0..steps {
        let a0 = start + span * i as f32 / steps as f32;
        let a1 = start + span * (i + 1) as f32 / steps as f32;
        draw_triangle(c, edge(a0), edge(a1), col);
    }
}

fn hud_slot(r: Rect, key: &str, icon: Option<&Texture2D>, frac: f32, wait: f32, extra: Option<String>, ready: bool, ult: bool) {
    if ready {
        let pulse = 0.55 + 0.45 * (get_time() as f32 * 4.0).sin().abs();
        let g = if ult { crate::ui::GOLD } else { alpha(crate::ui::GOLD, 0.85) };
        draw_rectangle(r.x - 6.0, r.y - 6.0, r.w + 12.0, r.h + 12.0, alpha(crate::ui::GOLD, 0.18 * pulse));
        draw_rectangle(r.x - 4.0, r.y - 4.0, r.w + 8.0, r.h + 8.0, g);
    }
    draw_rectangle(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, crate::ui::INK);
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.91, 0.85, 0.7, 1.0));
    draw_rectangle(r.x, r.y, r.w, r.h * 0.3, alpha(WHITE, 0.3));
    match icon {
        Some(tex) => {
            let c = if ready { WHITE } else { Color::new(0.6, 0.6, 0.66, 1.0) };
            crate::ui::skill_art(tex, r, c);
        }
        None => text_mid(key, r.x + r.w / 2.0, r.y + r.h / 2.0 + 8.0, 24.0, crate::ui::INK),
    }
    if frac > 0.0 {
        sweep(r, frac, Color::new(0.1, 0.08, 0.18, 0.62));
    }
    let ks = if r.w < 50.0 { 12 } else { 14 };
    crate::ui::keycap(key, r.x + r.w / 2.0, r.y - 10.0, ks);
    if wait > 0.0 {
        let s = secs(wait);
        let size = (r.w * 0.42).round();
        let w = measure_text(&s, None, size as u16, 1.0).width;
        crate::ui::text_out(&s, r.x + (r.w - w) / 2.0, r.y + r.h / 2.0 + size * 0.32, size as u16, WHITE, 1.5);
    }
    if let Some(e) = extra {
        let w = measure_text(&e, None, 16, 1.0).width + 8.0;
        draw_rectangle(r.x + r.w - w - 2.0, r.y + r.h - 18.0, w, 16.0, crate::ui::INK);
        draw_text(&e, r.x + r.w - w + 2.0, r.y + r.h - 5.0, 16.0, WHITE);
    }
}

fn passive_slot(p: &Player, r: Rect, label: bool) {
    let (frac, extra, on) = passive_state(p);
    let name = p.kit().passive.name();
    hud_slot(r, if label { name } else { "P" }, Some(assets::passive_icon(p.kit().passive)), frac, 0.0, extra, on, false);
}

fn skill_row(p: &Player, x: f32, y: f32, size: f32, gap: f32, right: bool, label: bool) {
    let keys = ["LMB", "Q", "E", "R"];
    let psize = (size * 0.86).round();
    let total = size * 4.0 + gap * 4.0 + 10.0 + psize;
    let at = |off: f32, w: f32| if right { x + total - off - w } else { x + off };
    for (k, key) in keys.iter().enumerate() {
        let off = k as f32 * (size + gap);
        let r = Rect::new(at(off, size), y, size, size);
        let extra = p.skill(k).charged().then(|| p.charges.to_string());
        hud_slot(r, key, assets::skill_icon(p.skill(k)), p.cooldown_frac(k), p.cd(k), extra, p.ready(k), k == 3);
    }
    let off = 4.0 * (size + gap) + 10.0;
    let sep = at(off - gap / 2.0 - 4.0, 2.0);
    let mut dy = y + 4.0;
    while dy < y + size - 4.0 {
        draw_rectangle(sep, dy, 2.0, 6.0, alpha(crate::ui::INK, 0.35));
        dy += 10.0;
    }
    passive_slot(p, Rect::new(at(off, psize), y + size - psize, psize, psize), label);
}

fn draw_dock(p: &Player, name: &str, lag: f32) {
    let d = DOCK;
    crate::ui::hud_panel(d, 0.5);
    let pr = Rect::new(d.x + 18.0, d.y + 17.0, 84.0, 84.0);
    crate::ui::portrait_box(p.kit, pr, crate::ui::MINT, false);
    let cx = pr.x + pr.w + 18.0;
    let cw = d.x + d.w - 20.0 - cx;
    draw_text(name, cx, d.y + 28.0, 20.0, crate::ui::INK);
    let nw = measure_text(name, None, 20, 1.0).width;
    draw_text(p.kit().name, cx + nw + 10.0, d.y + 28.0, 16.0, SUBTLE);
    let hp = format!("{} / {}", p.hp.max(0), p.max_hp());
    let hw = measure_text(&hp, None, 17, 1.0).width;
    draw_text(&hp, cx + cw - hw, d.y + 28.0, 17.0, crate::ui::INK);
    hp_bar(Rect::new(cx - 4.0, d.y + 32.0, cw + 8.0, 16.0), p, lag, 0, false);
    skill_row(p, cx, d.y + 60.0, 46.0, 14.0, false, true);
    status_chips(p, d.x + 18.0, d.y - 28.0, false);
}

fn draw_foe(p: &Player, name: &str, lag: f32) {
    let r = FOE;
    crate::ui::hud_panel(r, 0.45);
    let pr = Rect::new(r.x + r.w - 16.0 - 54.0, r.y + 14.0, 54.0, 54.0);
    crate::ui::portrait_box(p.kit, pr, crate::ui::ROSE, true);
    let right = pr.x - 12.0;
    let left = r.x + 18.0;
    let nw = measure_text(name, None, 18, 1.0).width;
    draw_text(name, right - nw, r.y + 28.0, 18.0, crate::ui::INK);
    let kw = measure_text(p.kit().name, None, 14, 1.0).width;
    draw_text(p.kit().name, right - nw - kw - 6.0, r.y + 28.0, 14.0, SUBTLE);
    draw_text(&format!("{}/{}", p.hp.max(0), p.max_hp()), left, r.y + 28.0, 15.0, crate::ui::INK);
    hp_bar(Rect::new(left - 4.0, r.y + 33.0, right - left + 8.0, 16.0), p, lag, 1, true);
    status_chips(p, right, r.y + 56.0, true);
}

fn draw_half(i: usize, p: &Player, tag: &str, lag: f32) {
    let r = half(i);
    let right = i == 1;
    crate::ui::hud_panel(r, 0.5);
    let ps = 76.0;
    let pr = Rect::new(if right { r.x + r.w - 18.0 - ps } else { r.x + 18.0 }, r.y + 22.0, ps, ps);
    crate::ui::portrait_box(p.kit, pr, if right { crate::ui::ROSE } else { crate::ui::MINT }, right);
    let (cx, cw) = if right { (r.x + 20.0, pr.x - 16.0 - r.x - 20.0) } else { (pr.x + ps + 16.0, r.x + r.w - 20.0 - pr.x - ps - 16.0) };
    let name = p.kit().name;
    let nw = measure_text(name, None, 20, 1.0).width;
    let tw = crate::ui::chip_w(tag, 13);
    let hp = format!("{}/{}", p.hp.max(0), p.max_hp());
    let hw = measure_text(&hp, None, 16, 1.0).width;
    if right {
        draw_text(name, cx + cw - nw, r.y + 28.0, 20.0, crate::ui::INK);
        crate::ui::chip(tag, cx + cw - nw - tw - 6.0, r.y + 13.0, 13, crate::ui::LAV_D, WHITE);
        draw_text(&hp, cx, r.y + 28.0, 16.0, crate::ui::INK);
    } else {
        draw_text(name, cx, r.y + 28.0, 20.0, crate::ui::INK);
        crate::ui::chip(tag, cx + nw + 6.0, r.y + 13.0, 13, crate::ui::LAV_D, WHITE);
        draw_text(&hp, cx + cw - hw, r.y + 28.0, 16.0, crate::ui::INK);
    }
    hp_bar(Rect::new(cx - 4.0, r.y + 33.0, cw + 8.0, 16.0), p, lag, i, right);
    let size: f32 = 42.0;
    let gap: f32 = 10.0;
    let total = size * 4.0 + gap * 4.0 + 10.0 + (size * 0.86).round();
    let sx = if right { cx + cw - total } else { cx };
    skill_row(p, sx, r.y + 62.0, size, gap, right, false);
    status_chips(p, if right { r.x + r.w - 16.0 } else { r.x + 16.0 }, r.y + r.h + 6.0, right);
}

fn draw_score(score: [u32; 2]) {
    let r = Rect::new(VIEW.x / 2.0 - 130.0, 8.0, 260.0, 92.0);
    crate::ui::hud_panel(r, 0.45);
    let t = format!("{}  -  {}", score[0], score[1]);
    let w = measure_text(&t, None, 44, 1.0).width;
    crate::ui::text_out(&t, (VIEW.x - w) / 2.0, r.y + 50.0, 44, WHITE, 2.0);
    let n = crate::core::DEATHS_TO_WIN;
    for side in 0..2 {
        for k in 0..n {
            let off = 18.0 + k as f32 * 20.0;
            let x = if side == 0 { VIEW.x / 2.0 - off } else { VIEW.x / 2.0 + off };
            let on = k < score[side];
            let c = if !on { alpha(crate::ui::INK, 0.2) } else if side == 0 { crate::ui::MINT_D } else { crate::ui::ROSE_D };
            draw_poly(x, r.y + 72.0, 4, 8.0, 0.0, crate::ui::INK);
            draw_poly(x, r.y + 72.0, 4, 6.0, 0.0, c);
        }
    }
}


fn secs(t: f32) -> String {
    if t < 1.0 {
        format!("{t:.1}")
    } else {
        format!("{}", t.ceil() as i32)
    }
}

fn text_mid(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = measure_text(t, None, size as u16, 1.0).width;
    draw_text(t, cx - w / 2.0, y, size, c);
}

pub fn score_rect() -> Rect {
    Rect::new(VIEW.x / 2.0 - 260.0, 108.0, 520.0, 56.0)
}

pub fn body_rect(cam: &Camera, p: &Player) -> Rect {
    let s = cam.proj(p.pos);
    Rect::new(s.x - 40.0, s.y - 120.0, 80.0, 132.0)
}

pub fn draw_hud(players: &[Player; 2], hud: &Hud, lag: [f32; 2]) {
    match hud {
        Hud::Focus { names, score } => {
            draw_foe(&players[1], &names[1], lag[1]);
            draw_dock(&players[0], &names[0], lag[0]);
            if let Some(s) = score {
                draw_score(*s);
            }
        }
        Hud::Equal { tags } => {
            for (i, p) in players.iter().enumerate() {
                draw_half(i, p, &tags[i], lag[i]);
            }
        }
    }
}

pub fn draw_cursor(cam: &Camera) {
    let m = cam.mouse_view();
    crate::ui::reticle(m, 2.0);
}

fn passive_state(p: &Player) -> (f32, Option<String>, bool) {
    use crate::core::kit::Passive;
    match p.kit().passive {
        Passive::Grace => (1.0 - p.haste / crate::core::player::GRACE_T, None, p.haste > 0.0),
        Passive::Hexed => (0.0, None, true),
        Passive::Intent => (0.0, Some(format!("{}/4", p.intent)), p.intent >= 3),
        Passive::Bond => (0.0, None, p.bond),
        Passive::Fox => (0.0, None, !p.armed),
        Passive::Overcharge => (1.0 - p.energy / crate::core::seren::ENERGY_MAX, Some(format!("{:.0}", p.energy)), p.energy >= crate::core::seren::ENERGY_MAX || !p.armed),
    }
}

const LOW_ENERGY: f32 = 0.3;

fn energy_glow(p: &Player) -> Option<bool> {
    if p.kit().passive != crate::core::kit::Passive::Overcharge {
        return None;
    }
    let k = p.energy / crate::core::seren::ENERGY_MAX;
    if p.armed && k >= 1.0 {
        Some(true)
    } else if !p.armed && k < LOW_ENERGY {
        Some(false)
    } else {
        None
    }
}
