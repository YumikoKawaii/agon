use std::f32::consts::{PI, TAU};

use macroquad::{prelude::*, rand::gen_range};

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
pub const CRIMSON: Color = Color::new(1.0, 0.22, 0.3, 1.0);
pub const STEEL: Color = Color::new(0.8, 0.85, 0.92, 1.0);
pub const SAKURA: Color = Color::new(1.0, 0.68, 0.84, 1.0);
const FLOOR: [Color; 2] = [Color::new(0.27, 0.33, 0.34, 1.0), Color::new(0.3, 0.37, 0.37, 1.0)];
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
    let (nx, ny) = ((ARENA.x / TILE) as i32, (ARENA.y / TILE) as i32);
    for i in 0..nx {
        for j in 0..ny {
            let p = cam.proj(vec2(i as f32 * TILE, j as f32 * TILE));
            draw_rectangle(p.x, p.y, TILE * S, TILE * S, FLOOR[((i + j) % 2) as usize]);
        }
    }
}

pub fn draw_cage(cam: &Camera, front: bool) {
    let (nx, ny) = ((ARENA.x / TILE) as i32, (ARENA.y / TILE) as i32);
    let along_x = (0..=nx).flat_map(|k| [vec2(k as f32 * TILE, 0.0), vec2(k as f32 * TILE, ARENA.y)]);
    let along_y = (0..=ny).flat_map(|k| [vec2(0.0, k as f32 * TILE), vec2(ARENA.x, k as f32 * TILE)]);
    for w in along_x.chain(along_y) {
        if (w.y == ARENA.y) == front {
            let p = cam.proj(w);
            draw_line(p.x, p.y, p.x, p.y - POST_H * S, 2.0, CAGE);
        }
    }
    let (w, h) = (ARENA.x, ARENA.y);
    let back = [(vec2(0.0, 0.0), vec2(w, 0.0)), (vec2(0.0, 0.0), vec2(0.0, h)), (vec2(w, 0.0), vec2(w, h))];
    let fore = [(vec2(0.0, h), vec2(w, h))];
    let edges: &[(Vec2, Vec2)] = if front { &fore } else { &back };
    for (a, b) in edges {
        let (a, b) = (cam.proj(*a), cam.proj(*b));
        for h in [POST_H, POST_H / 2.0] {
            draw_line(a.x, a.y - h * S, b.x, b.y - h * S, 2.0, CAGE);
        }
    }
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
    let skin = |armed: bool| if lit && !armed { &assets::get().seren_lit } else { assets::skin_in(p.kit, armed) };
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

const PLATE: Vec2 = vec2(470.0, 96.0);
const HP_FILL: [Color; 2] = [Color::new(0.5, 0.84, 0.76, 1.0), Color::new(0.91, 0.46, 0.54, 1.0)];

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

fn draw_plate(i: usize, p: &Player) {
    let right = i == 1;
    let x = if right { VIEW.x - 20.0 - PLATE.x } else { 20.0 };
    let r = Rect::new(x, 6.0, PLATE.x, PLATE.y);
    crate::ui::panel_scaled(r, 0.5);
    let pr = Rect::new(if right { r.x + r.w - 92.0 } else { r.x + 14.0 }, r.y + 12.0, 78.0, 72.0);
    draw_rectangle(pr.x + 4.0, pr.y + 4.0, pr.w - 8.0, pr.h - 8.0, Color::new(0.82, 0.9, 0.86, 1.0));
    let tex = assets::portrait(p.kit);
    let side = pr.h - 8.0;
    draw_texture_ex(tex, pr.x + (pr.w - side) / 2.0, pr.y + 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(side, side)), flip_x: right, ..Default::default() });
    crate::ui::frame_border(pr);
    let bx = if right { r.x + 18.0 } else { pr.x + pr.w + 10.0 };
    let bw = r.w - pr.w - 44.0;
    let name = p.kit().name;
    let hp = format!("{}/{}", p.hp.max(0), p.max_hp());
    let (nl, hl) = (measure_text(name, None, 22, 1.0).width, measure_text(&hp, None, 20, 1.0).width);
    let (nx, hx) = if right { (bx + bw - nl, bx) } else { (bx, bx + bw - hl) };
    draw_text(name, nx, r.y + 32.0, 22.0, crate::ui::INK);
    draw_text(&hp, hx, r.y + 32.0, 20.0, crate::ui::INK);
    crate::ui::bar(Rect::new(bx, r.y + 38.0, bw, 22.0), p.hp.max(0) as f32 / p.max_hp() as f32, HP_FILL[i], right);
    let mut n = 0.0;
    for k in KINDS.iter().copied().filter(|k| p.effects.on(*k)) {
        let label = match k {
            EffectKind::Bleed => format!("{} x{}", status_name(k), p.effects.stacks),
            EffectKind::Shield => format!("{} {}", status_name(k), p.effects.power[k.i()]),
            _ => status_name(k).to_string(),
        };
        let tw = measure_text(&label, None, 16, 1.0).width + 12.0;
        let ex = if right { bx + bw - n - tw } else { bx + n };
        let col = status_color(k);
        draw_rectangle(ex, r.y + 66.0, tw, 18.0, crate::ui::INK);
        draw_rectangle(ex + 1.0, r.y + 82.0, (tw - 2.0) * (1.0 - p.effects.progress(k)), 2.0, col);
        draw_text(&label, ex + 6.0, r.y + 80.0, 16.0, col);
        n += tw + 4.0;
    }
}

fn draw_slot(r: Rect, key: &str, icon: Option<&Texture2D>, tint: Color, frac: f32, wait: f32, extra: Option<String>, ready: bool) {
    let t = if ready { tint } else { Color::new(tint.r * 0.6, tint.g * 0.6, tint.b * 0.65, 1.0) };
    crate::ui::slot(r, t);
    match icon {
        Some(tex) => {
            let s = r.w - 18.0;
            let c = if ready { WHITE } else { Color::new(0.55, 0.55, 0.6, 1.0) };
            draw_texture_ex(tex, r.x + 9.0, r.y + 9.0, c, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
        }
        None => text_mid(key, r.x + r.w / 2.0, r.y + r.h / 2.0 + 8.0, 24.0, if ready { crate::ui::INK } else { WHITE }),
    }
    if frac > 0.0 {
        let h = (r.h - 12.0) * frac.clamp(0.0, 1.0);
        draw_rectangle(r.x + 6.0, r.y + 6.0, r.w - 12.0, h, Color::new(0.1, 0.08, 0.18, 0.55));
    }
    let kw = measure_text(key, None, 14, 1.0).width + 8.0;
    draw_rectangle(r.x + (r.w - kw) / 2.0, r.y - 9.0, kw, 16.0, crate::ui::INK);
    text_mid(key, r.x + r.w / 2.0, r.y + 3.0, 14.0, WHITE);
    if wait > 0.0 {
        let s = secs(wait);
        text_mid(&s, r.x + r.w / 2.0 + 1.0, r.y + r.h / 2.0 + 11.0, 30.0, crate::ui::INK);
        text_mid(&s, r.x + r.w / 2.0, r.y + r.h / 2.0 + 10.0, 30.0, WHITE);
    }
    if let Some(e) = extra {
        let w = measure_text(&e, None, 16, 1.0).width + 8.0;
        draw_rectangle(r.x + r.w - w - 2.0, r.y + r.h - 18.0, w, 16.0, crate::ui::INK);
        draw_text(&e, r.x + r.w - w + 2.0, r.y + r.h - 5.0, 16.0, WHITE);
    }
}

fn draw_skills(p: &Player) {
    let sizes = [70.0, 70.0, 70.0, 84.0, 62.0];
    let gap = 12.0;
    let total: f32 = sizes.iter().sum::<f32>() + gap * 5.0 + 40.0;
    let r = Rect::new((VIEW.x - total) / 2.0, VIEW.y - 112.0, total, 104.0);
    crate::ui::panel_scaled(r, 0.5);
    let lav = Color::new(1.0, 1.0, 1.0, 1.0);
    let gold = Color::new(1.0, 0.9, 0.55, 1.0);
    let mint = Color::new(0.7, 1.15, 1.0, 1.0);
    let mut x = r.x + 20.0;
    for (k, key) in ["LMB", "Q", "E", "R"].iter().enumerate() {
        let s = sizes[k];
        let sr = Rect::new(x, r.y + r.h - 16.0 - s, s, s);
        let extra = if p.skill(k).charged() {
            Some(p.charges.to_string())
        } else if p.skill(k) == crate::core::Skill::Cable {
            Some(if p.fury > 0.0 { "100".to_string() } else { (p.gas as i32).to_string() })
        } else {
            None
        };
        draw_slot(sr, key, assets::skill_icon(p.skill(k)), if k == 3 { gold } else { lav }, p.cooldown_frac(k), p.cd(k), extra, p.ready(k));
        x += s + gap;
        if k == 3 {
            draw_rectangle(x - gap / 2.0 - 1.0, r.y + 22.0, 2.0, r.h - 40.0, alpha(crate::ui::INK, 0.4));
            x += gap;
        }
    }
    let s = sizes[4];
    let sr = Rect::new(x, r.y + r.h - 16.0 - s, s, s);
    let (frac, extra, on) = passive_state(p);
    draw_slot(sr, "P", Some(assets::passive_icon(p.kit().passive)), mint, frac, 0.0, extra, on);
    text_mid(p.kit().passive.name(), sr.x + s / 2.0, r.y + 18.0, 14.0, crate::ui::INK);
}

pub fn score_rect() -> Rect {
    Rect::new(VIEW.x / 2.0 - 260.0, 108.0, 520.0, 56.0)
}

pub fn hud_zones() -> [Rect; 3] {
    let sizes = 70.0 * 3.0 + 84.0 + 62.0 + 12.0 * 5.0 + 40.0;
    [Rect::new(0.0, 0.0, VIEW.x, PLATE.y + 12.0), Rect::new((VIEW.x - sizes) / 2.0, VIEW.y - 124.0, sizes, 124.0), score_rect()]
}

pub fn body_rect(cam: &Camera, p: &Player) -> Rect {
    let s = cam.proj(p.pos);
    Rect::new(s.x - 40.0, s.y - 120.0, 80.0, 132.0)
}

pub fn draw_hud(players: &[Player; 2], title: Option<&str>) {
    for (i, p) in players.iter().enumerate() {
        draw_plate(i, p);
    }
    if let Some(title) = title {
        let tw = measure_text(title, None, 24, 1.0).width.max(120.0) + 60.0;
        let tr = Rect::new((VIEW.x - tw) / 2.0, 14.0, tw, 52.0);
        crate::ui::panel_scaled(tr, 0.4);
        text_mid(title, VIEW.x / 2.0, tr.y + 34.0, 24.0, crate::ui::INK);
    }
    draw_skills(&players[0]);
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
        Passive::Overcharge => (0.0, Some(format!("{}/{}", p.over, crate::core::seren::OVER_MAX)), p.over >= crate::core::seren::OVER_MAX),
    }
}
