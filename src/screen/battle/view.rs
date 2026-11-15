use std::f32::consts::{PI, TAU};

use macroquad::{prelude::*, rand::gen_range};

use crate::{
    assets,
    core::{anim::State, Player, ARENA, RADIUS},
};

pub const S: f32 = 3.0;
pub const SPRITE: f32 = 0.5;
pub const FOOT: f32 = 9.0;
pub const CHEST: f32 = 9.0;
const POST_H: f32 = 14.0;
const TILE: f32 = 20.0;
const CAM_MARGIN: f32 = 40.0;
const HIT_FLASH_TICKS: u32 = 4;

pub const ICE: Color = Color::new(0.55, 0.85, 1.0, 1.0);
pub const PALE: Color = Color::new(0.88, 0.97, 1.0, 1.0);
pub const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);
pub const AQUA: Color = Color::new(0.45, 1.0, 0.92, 1.0);
pub const SEAT_COLORS: [Color; 2] = [SKYBLUE, ORANGE];
const FLOOR: [Color; 2] = [Color::new(0.2, 0.22, 0.29, 1.0), Color::new(0.227, 0.247, 0.322, 1.0)];
const CAGE: Color = Color::new(0.541, 0.565, 0.651, 1.0);
const HIT_TINT: Color = Color::new(1.0, 0.6, 0.6, 1.0);

#[derive(Default)]
pub struct Camera {
    offset: Vec2,
}

impl Camera {
    pub fn follow(&mut self, target: Vec2, shake: f32) {
        let half = vec2(screen_width(), screen_height()) / 2.0;
        let lo = vec2(-CAM_MARGIN, -POST_H * S - CAM_MARGIN);
        let hi = Vec2::splat(ARENA * S + CAM_MARGIN) - half * 2.0;
        let jitter = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)) * shake;
        let t = target * S - half + jitter;
        self.offset = vec2(t.x.clamp(lo.x, hi.x.max(lo.x)), t.y.clamp(lo.y, hi.y.max(lo.y)));
    }

    pub fn proj(&self, v: Vec2) -> Vec2 {
        v * S - self.offset
    }

    pub fn air(&self, v: Vec2, z: f32) -> Vec2 {
        self.proj(v) - vec2(0.0, z * S)
    }

    pub fn mouse_world(&self) -> Vec2 {
        (Vec2::from(mouse_position()) + self.offset + vec2(0.0, FOOT * S)) / S
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
    let n = (ARENA / TILE) as i32;
    for i in 0..n {
        for j in 0..n {
            let p = cam.proj(vec2(i as f32 * TILE, j as f32 * TILE));
            draw_rectangle(p.x, p.y, TILE * S, TILE * S, FLOOR[((i + j) % 2) as usize]);
        }
    }
}

pub fn draw_cage(cam: &Camera, front: bool) {
    let n = (ARENA / TILE) as i32;
    for k in 0..=n {
        let t = k as f32 * TILE;
        for w in [vec2(t, 0.0), vec2(t, ARENA), vec2(0.0, t), vec2(ARENA, t)] {
            if (w.y == ARENA) == front {
                let p = cam.proj(w);
                draw_line(p.x, p.y, p.x, p.y - POST_H * S, 2.0, CAGE);
            }
        }
    }
    let back = [(vec2(0.0, 0.0), vec2(ARENA, 0.0)), (vec2(0.0, 0.0), vec2(0.0, ARENA)), (vec2(ARENA, 0.0), vec2(ARENA, ARENA))];
    let fore = [(vec2(0.0, ARENA), vec2(ARENA, ARENA))];
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
    let tex = assets::skin(p.kit).frame(p.state, p.dir8(), p.age, p.anim_ticks());
    let tint = if p.state == State::Hit && p.age < HIT_FLASH_TICKS { HIT_TINT } else { WHITE };
    draw_sprite(tex, s, tint);
    if debug {
        draw_text(&format!("{:?} {} d{}", p.state, p.frame, p.dir8()), s.x - 40.0, s.y - 22.0 * S, 20.0, WHITE);
    }
}

pub fn draw_hud(players: &[Player; 2]) {
    let w = 300.0;
    let slot = Color::from_rgba(34, 38, 52, 255);
    for (i, p) in players.iter().enumerate() {
        let x = if i == 0 { 40.0 } else { screen_width() - 40.0 - w };
        draw_rectangle(x, 24.0, w, 14.0, slot);
        let fill = w * p.hp.max(0) as f32 / p.max_hp() as f32;
        draw_rectangle(if i == 0 { x } else { x + w - fill }, 24.0, fill, 14.0, SEAT_COLORS[i]);
        let label = format!("{}  {}/{}", p.kit().name, p.hp.max(0), p.max_hp());
        let lx = if i == 0 { x } else { x + w - measure_text(&label, None, 18, 1.0).width };
        draw_text(&label, lx, 18.0, 18.0, LIGHTGRAY);
        for (k, key) in ["LMB", "Q", "E", "R"].iter().enumerate() {
            let bx = if i == 0 { x + k as f32 * 52.0 } else { x + w - 48.0 - k as f32 * 52.0 };
            draw_rectangle(bx, 44.0, 48.0, 22.0, slot);
            let col = if p.ready(k) { ICE } else { Color::from_rgba(70, 76, 98, 255) };
            draw_rectangle(bx, 44.0, 48.0 * (1.0 - p.cooldown_frac(k)), 22.0, col);
            let text = if p.kit().skills[k].charged() { format!("{key} {}", p.charges) } else { key.to_string() };
            draw_text(&text, bx + 6.0, 61.0, 20.0, Color::from_rgba(20, 20, 28, 255));
        }
    }
}

pub fn draw_cursor() {
    let m = Vec2::from(mouse_position());
    draw_circle_lines(m.x, m.y, 8.0, 1.5, WHITE);
    draw_circle(m.x, m.y, 2.0, WHITE);
}
