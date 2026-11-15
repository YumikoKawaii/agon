use crate::anim::Anims;
use crate::player::*;
use crate::{proj, S};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

pub const ICE: Color = Color::new(0.55, 0.85, 1.0, 1.0);
pub const PALE: Color = Color::new(0.88, 0.97, 1.0, 1.0);
pub const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);
pub const ROSE: Color = Color::new(1.0, 0.55, 0.75, 1.0);

pub const JUDGE_RANGE: f32 = 130.0;
pub const RAY_LEN: f32 = 280.0;
const JUDGE_R: f32 = 42.0;
const JUDGE_T: f32 = 1.0;
const RAY_CHARGE: f32 = 0.4;
const RAY_SHOW: f32 = 0.3;
const RAY_W: f32 = 7.0;
const WARD_T: f32 = 1.5;
const WARD_W: f32 = 22.0;
const WARD_OFF: f32 = 14.0;
const CHEST: f32 = 9.0;

fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a.clamp(0.0, 1.0))
}

fn glow(c: Vec2, r: f32, col: Color, a: f32) {
    draw_circle(c.x, c.y, r * 2.4, alpha(col, a * 0.12));
    draw_circle(c.x, c.y, r * 1.5, alpha(col, a * 0.3));
    draw_circle(c.x, c.y, r, alpha(col, a));
    draw_circle(c.x, c.y, r * 0.45, alpha(WHITE, a));
}

fn air(p: Vec2, z: f32) -> Vec2 {
    proj(p) - vec2(0.0, z * S)
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(0.0001)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

struct Part {
    pos: Vec2,
    z: f32,
    vel: Vec2,
    vz: f32,
    life: f32,
    max: f32,
    size: f32,
    col: Color,
    drag: f32,
}

struct Shot {
    pos: Vec2,
    vel: Vec2,
    r: f32,
    dmg: i32,
    owner: usize,
    life: f32,
    col: Color,
}

struct Ray {
    origin: Vec2,
    dir: Vec2,
    t: f32,
    owner: usize,
    fired: bool,
}

struct Ward {
    owner: usize,
    t: f32,
}

struct Judge {
    pos: Vec2,
    t: f32,
    owner: usize,
}

struct Pillar {
    pos: Vec2,
    t: f32,
}

struct Ring {
    pos: Vec2,
    z: f32,
    r0: f32,
    r1: f32,
    t: f32,
    max: f32,
    col: Color,
    w: f32,
}

struct Num {
    pos: Vec2,
    t: f32,
    dmg: i32,
}

#[derive(Default)]
pub struct World {
    shots: Vec<Shot>,
    rays: Vec<Ray>,
    wards: Vec<Ward>,
    judges: Vec<Judge>,
    pillars: Vec<Pillar>,
    parts: Vec<Part>,
    rings: Vec<Ring>,
    nums: Vec<Num>,
    pub shake: f32,
    pub flash: f32,
}

impl World {
    fn part(&mut self, pos: Vec2, z: f32, vel: Vec2, vz: f32, life: f32, size: f32, col: Color, drag: f32) {
        self.parts.push(Part { pos, z, vel, vz, life, max: life, size, col, drag });
    }

    fn burst(&mut self, pos: Vec2, z: f32, n: usize, speed: f32, life: f32, size: f32, col: Color) {
        for _ in 0..n {
            let a = gen_range(0.0, TAU);
            let s = gen_range(speed * 0.3, speed);
            let vz = gen_range(-speed * 0.3, speed * 0.6);
            self.part(pos, z, Vec2::from_angle(a) * s, vz, gen_range(life * 0.5, life), gen_range(size * 0.5, size), col, 0.9);
        }
    }

    fn ring(&mut self, pos: Vec2, z: f32, r0: f32, r1: f32, max: f32, col: Color, w: f32) {
        self.rings.push(Ring { pos, z, r0, r1, t: max, max, col, w });
    }

    fn hit_fx(&mut self, pos: Vec2, col: Color, dmg: i32, big: bool) {
        let n = if big { 30 } else { 10 };
        self.burst(pos, CHEST, n, if big { 140.0 } else { 80.0 }, 0.45, if big { 3.0 } else { 2.0 }, col);
        self.burst(pos, CHEST, n / 2, 60.0, 0.3, 1.5, WHITE);
        self.ring(pos, CHEST, 2.0, if big { 26.0 } else { 12.0 }, 0.3, col, 3.0);
        self.nums.push(Num { pos: pos + vec2(gen_range(-4.0, 4.0), 0.0), t: 0.8, dmg });
        self.shake = self.shake.max(if big { 7.0 } else { 3.0 });
    }

    pub fn enemy_shot(&mut self, from: Vec2, to: Vec2, owner: usize) {
        let d = (to - from).normalize_or(Vec2::X);
        self.shots.push(Shot { pos: from + d * 8.0, vel: d * 150.0, r: 3.5, dmg: 5, owner, life: 2.5, col: ROSE });
    }

    pub fn cast(&mut self, p: &mut Player, i: usize) {
        let Some(k) = p.cast.take() else { return };
        let f = p.facing();
        let tip = p.pos + f * 8.0;
        match k {
            Skill::Needle => {
                self.shots.push(Shot { pos: tip, vel: f * 320.0, r: 2.5, dmg: 6, owner: i, life: 0.8, col: ICE });
                for _ in 0..6 {
                    let v = Vec2::from_angle(p.ang + gen_range(-0.6, 0.6)) * gen_range(40.0, 110.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 20.0), 0.25, 1.5, PALE, 0.85);
                }
                self.ring(tip, CHEST, 1.0, 7.0, 0.15, ICE, 2.0);
            }
            Skill::Ray => self.rays.push(Ray { origin: p.pos, dir: f, t: 0.0, owner: i, fired: false }),
            Skill::Ward => {
                self.wards.push(Ward { owner: i, t: 0.0 });
                self.ring(p.pos, CHEST, 4.0, 30.0, 0.35, PALE, 3.0);
                self.burst(p.pos + f * WARD_OFF, CHEST, 16, 60.0, 0.4, 2.0, ICE);
            }
            Skill::Judgement => {
                let m = JUDGE_R * 0.6;
                let at = (p.pos + (p.target - p.pos).clamp_length_max(JUDGE_RANGE)).clamp(vec2(m, m), vec2(ARENA - m, ARENA - m));
                self.judges.push(Judge { pos: at, t: 0.0, owner: i });
                self.ring(at, 0.0, JUDGE_R * 1.6, JUDGE_R, 0.4, GOLD, 3.0);
                self.burst(p.pos, CHEST + 8.0, 20, 70.0, 0.5, 2.0, GOLD);
            }
        }
    }

    fn ward_seg(p: &Player) -> (Vec2, Vec2) {
        let f = p.facing();
        let c = p.pos + f * WARD_OFF;
        (c - f.perp() * WARD_W, c + f.perp() * WARD_W)
    }

    pub fn update(&mut self, players: &mut [Player; 2], anims: &Anims) {
        let mut shots = std::mem::take(&mut self.shots);
        shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            self.part(s.pos, CHEST + gen_range(-1.0, 1.0), -s.vel * 0.1, 0.0, 0.25, s.r * 0.7, s.col, 0.9);
            let blocked = self.wards.iter().find(|w| w.owner != s.owner).map(|w| w.owner).filter(|&o| {
                let (a, b) = Self::ward_seg(&players[o]);
                seg_dist(s.pos, a, b) < s.r + 3.0
            });
            if let Some(o) = blocked {
                players[o].cd[2] = (players[o].cd[2] - 2.0).max(0.0);
                self.burst(s.pos, CHEST, 18, 120.0, 0.35, 2.0, PALE);
                self.ring(s.pos, CHEST, 2.0, 16.0, 0.25, ICE, 3.0);
                self.shake = self.shake.max(2.5);
                return false;
            }
            let t = 1 - s.owner;
            let hit = players[t].hurtboxes(anims).iter().any(|(c, r)| c.distance(s.pos) < r + s.r);
            if hit {
                players[t].take_hit(s.vel.normalize_or_zero(), s.dmg);
                self.hit_fx(s.pos, s.col, s.dmg, false);
            }
            let inside = s.pos.cmpge(Vec2::ZERO).all() && s.pos.cmple(Vec2::splat(ARENA)).all();
            !hit && s.life > 0.0 && inside
        });
        self.shots = shots;

        let mut rays = std::mem::take(&mut self.rays);
        rays.retain_mut(|r| {
            r.t += DT;
            if !r.fired {
                r.origin = players[r.owner].pos;
                let tip = r.origin + r.dir * 8.0;
                for _ in 0..3 {
                    let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(14.0, 26.0);
                    self.part(tip + off, CHEST + gen_range(-8.0, 8.0), -off * 4.0, 0.0, 0.22, 1.8, if gen_range(0, 2) == 0 { ICE } else { PALE }, 1.0);
                }
            }
            if r.t >= RAY_CHARGE && !r.fired {
                r.fired = true;
                let end = r.origin + r.dir * RAY_LEN;
                let t = 1 - r.owner;
                if !players[t].hurtboxes(anims).is_empty() && seg_dist(players[t].pos, r.origin, end) < RAY_W + RADIUS {
                    players[t].take_hit(r.dir * 2.5, 22);
                    let at = players[t].pos;
                    self.hit_fx(at, ICE, 22, true);
                }
                for _ in 0..50 {
                    let along = r.origin + r.dir * gen_range(10.0, RAY_LEN);
                    let v = r.dir.perp() * gen_range(-90.0, 90.0) + r.dir * gen_range(20.0, 120.0);
                    self.part(along, CHEST, v, gen_range(-20.0, 40.0), gen_range(0.2, 0.5), gen_range(1.0, 2.5), if gen_range(0, 3) == 0 { WHITE } else { ICE }, 0.88);
                }
                let tip = r.origin + r.dir * 8.0;
                self.ring(tip, CHEST, 3.0, 22.0, 0.25, PALE, 4.0);
                self.shake = self.shake.max(6.0);
                self.flash = self.flash.max(0.12);
            }
            r.t < RAY_CHARGE + RAY_SHOW
        });
        self.rays = rays;

        let mut wards = std::mem::take(&mut self.wards);
        wards.retain_mut(|w| {
            w.t += DT;
            if gen_range(0, 3) == 0 {
                let (a, b) = Self::ward_seg(&players[w.owner]);
                let p = a.lerp(b, gen_range(0.0, 1.0));
                self.part(p, CHEST + gen_range(-6.0, 6.0), Vec2::ZERO, 18.0, 0.4, 1.3, PALE, 1.0);
            }
            w.t < WARD_T
        });
        self.wards = wards;

        let mut judges = std::mem::take(&mut self.judges);
        judges.retain_mut(|j| {
            j.t += DT;
            let k = (j.t / JUDGE_T).min(1.0);
            for _ in 0..(1 + (k * 4.0) as usize) {
                let a = gen_range(0.0, TAU);
                let p = j.pos + Vec2::from_angle(a) * JUDGE_R * gen_range(0.85, 1.0);
                self.part(p, 0.0, Vec2::ZERO, gen_range(40.0, 120.0), 0.5, gen_range(1.0, 2.2), if gen_range(0, 2) == 0 { GOLD } else { PALE }, 1.0);
            }
            if j.t < JUDGE_T {
                return true;
            }
            let t = 1 - j.owner;
            let d = players[t].pos - j.pos;
            if !players[t].hurtboxes(anims).is_empty() && d.length() < JUDGE_R + RADIUS {
                players[t].take_hit(d.normalize_or(Vec2::X) * 4.0, 35);
                let at = players[t].pos;
                self.hit_fx(at, GOLD, 35, true);
            }
            self.pillars.push(Pillar { pos: j.pos, t: 0.5 });
            self.ring(j.pos, 0.0, 4.0, JUDGE_R * 1.8, 0.5, GOLD, 6.0);
            self.ring(j.pos, 0.0, 4.0, JUDGE_R * 1.3, 0.35, WHITE, 4.0);
            self.ring(j.pos, CHEST, 4.0, JUDGE_R, 0.25, PALE, 3.0);
            for _ in 0..90 {
                let a = gen_range(0.0, TAU);
                let v = Vec2::from_angle(a) * gen_range(60.0, 220.0);
                self.part(j.pos, gen_range(0.0, 10.0), v, gen_range(20.0, 200.0), gen_range(0.4, 0.9), gen_range(1.5, 3.5), if gen_range(0, 3) == 0 { WHITE } else { GOLD }, 0.92);
            }
            self.shake = self.shake.max(14.0);
            self.flash = self.flash.max(0.35);
            false
        });
        self.judges = judges;

        self.parts.retain_mut(|p| {
            p.pos += p.vel * DT;
            p.z = (p.z + p.vz * DT).max(0.0);
            p.vel *= p.drag;
            p.vz *= p.drag;
            p.life -= DT;
            p.life > 0.0
        });
        self.rings.retain_mut(|r| {
            r.t -= DT;
            r.t > 0.0
        });
        self.pillars.retain_mut(|p| {
            p.t -= DT;
            p.t > 0.0
        });
        self.nums.retain_mut(|n| {
            n.t -= DT;
            n.t > 0.0
        });
        self.shake *= 0.86;
        self.flash = (self.flash - DT).max(0.0);
    }

    fn magic_circle(c: Vec2, r: f32, rot: f32, col: Color, a: f32, w: f32) {
        draw_circle_lines(c.x, c.y, r, w, alpha(col, a));
        draw_circle_lines(c.x, c.y, r * 0.82, w * 0.6, alpha(col, a * 0.8));
        draw_circle_lines(c.x, c.y, r * 0.4, w * 0.6, alpha(col, a * 0.7));
        for tri in 0..2 {
            let base = rot + tri as f32 * PI / 3.0;
            let v: Vec<Vec2> = (0..3).map(|i| c + Vec2::from_angle(base + i as f32 * TAU / 3.0) * r * 0.82).collect();
            draw_triangle_lines(v[0], v[1], v[2], w * 0.7, alpha(col, a * 0.9));
        }
        for i in 0..24 {
            let ang = -rot * 1.5 + i as f32 * TAU / 24.0;
            let d = Vec2::from_angle(ang);
            let len = if i % 3 == 0 { 0.1 } else { 0.05 };
            let p0 = c + d * r * 0.86;
            let p1 = c + d * r * (0.86 + len);
            draw_line(p0.x, p0.y, p1.x, p1.y, w * 0.6, alpha(col, a));
        }
        for i in 0..6 {
            let p = c + Vec2::from_angle(rot + i as f32 * TAU / 6.0) * r * 0.61;
            draw_poly_lines(p.x, p.y, 4, r * 0.07, rot.to_degrees() * 2.0, w * 0.5, alpha(col, a));
        }
    }

    pub fn draw_ground(&self) {
        let time = get_time() as f32;
        for j in &self.judges {
            let k = (j.t / JUDGE_T).min(1.0);
            let open = (k * 3.0).min(1.0);
            let c = proj(j.pos);
            let r = JUDGE_R * S * (0.3 + 0.7 * open);
            draw_circle(c.x, c.y, r, alpha(GOLD, 0.06 + 0.22 * k));
            Self::magic_circle(c, r, time * (1.0 + k * 3.0), GOLD, 0.5 + 0.5 * k, 3.0);
            let pulse = (k * 6.0 * TAU).sin().abs();
            draw_circle_lines(c.x, c.y, r * (1.0 - k), 2.0, alpha(WHITE, 0.4 + 0.4 * pulse));
        }
        for s in &self.shots {
            let c = proj(s.pos);
            draw_circle(c.x, c.y, s.r * S * 0.8, Color::new(0.0, 0.0, 0.0, 0.3));
        }
        for r in &self.rings {
            if r.z > 0.0 {
                continue;
            }
            let k = r.t / r.max;
            let c = proj(r.pos);
            let rr = (r.r1 + (r.r0 - r.r1) * k) * S;
            draw_circle_lines(c.x, c.y, rr, r.w * (0.5 + k), alpha(r.col, k));
        }
    }

    pub fn draw_air(&self, players: &[Player; 2]) {
        let time = get_time() as f32;
        for w in &self.wards {
            let p = &players[w.owner];
            let fade = (w.t / 0.1).min(1.0) * if w.t > WARD_T - 0.4 { if (w.t * 30.0) as i32 % 2 == 0 { 0.3 } else { 1.0 } } else { 1.0 };
            let f = p.facing();
            let center = p.pos + f * WARD_OFF;
            for i in -1..=1 {
                let cell = air(center + f.perp() * i as f32 * 14.5, CHEST);
                let pulse = 0.7 + 0.3 * (time * 6.0 + i as f32).sin();
                draw_poly(cell.x, cell.y, 6, 8.5 * S, 30.0, alpha(ICE, 0.18 * fade * pulse));
                draw_poly_lines(cell.x, cell.y, 6, 8.5 * S, 30.0, 2.0, alpha(PALE, 0.9 * fade));
                draw_poly_lines(cell.x, cell.y, 6, 5.0 * S, 30.0 + time * 90.0, 1.0, alpha(ICE, 0.6 * fade));
            }
        }
        for s in &self.shots {
            let c = air(s.pos, CHEST);
            let tail = air(s.pos - s.vel.normalize_or_zero() * s.r * 4.0, CHEST);
            draw_line(tail.x, tail.y, c.x, c.y, s.r * S * 1.2, alpha(s.col, 0.5));
            glow(c, s.r * S, s.col, 1.0);
        }
        for p in &self.parts {
            let k = p.life / p.max;
            let c = air(p.pos, p.z);
            draw_circle(c.x, c.y, p.size * S * 1.8, alpha(p.col, k * 0.2));
            draw_circle(c.x, c.y, p.size * S * (0.5 + 0.5 * k), alpha(p.col, k));
        }
        for r in &self.rays {
            let a = air(r.origin + r.dir * 8.0, CHEST);
            let b = air(r.origin + r.dir * RAY_LEN, CHEST);
            if !r.fired {
                let k = r.t / RAY_CHARGE;
                let flick = if (time * 40.0) as i32 % 2 == 0 { 1.0 } else { 0.6 };
                draw_line(a.x, a.y, b.x, b.y, 1.0 + k * 2.0, alpha(ICE, 0.25 * flick));
                Self::magic_circle(a, (6.0 + 6.0 * k) * S, time * 6.0, ICE, 0.9, 2.0);
                glow(a, (2.0 + 3.0 * k) * S, PALE, 0.9);
            } else {
                let k = 1.0 - (r.t - RAY_CHARGE) / RAY_SHOW;
                let w = RAY_W * S * (0.3 + 0.9 * k) * gen_range(0.85, 1.15);
                draw_line(a.x, a.y, b.x, b.y, w * 2.6, alpha(ICE, 0.18 * k));
                draw_line(a.x, a.y, b.x, b.y, w * 1.5, alpha(ICE, 0.55 * k));
                draw_line(a.x, a.y, b.x, b.y, w * 0.7, alpha(PALE, k));
                draw_line(a.x, a.y, b.x, b.y, w * 0.3, alpha(WHITE, k));
                glow(a, w * 0.9, PALE, k);
                glow(b, w * 0.6, ICE, k);
            }
        }
        for j in &self.judges {
            let k = (j.t / JUDGE_T).min(1.0);
            let c = air(j.pos, 70.0);
            let r = JUDGE_R * S * 0.7 * (k * 2.0).min(1.0);
            Self::magic_circle(c, r, -time * 2.0, PALE, 0.35 + 0.5 * k, 2.0);
            if k > 0.6 {
                let g = proj(j.pos);
                let kk = (k - 0.6) / 0.4;
                draw_line(c.x, c.y, g.x, g.y, 1.0 + kk * 4.0, alpha(PALE, 0.5 * kk));
            }
        }
        for p in &self.pillars {
            let k = p.t / 0.5;
            let g = proj(p.pos);
            let top = g.y - 900.0;
            let w = JUDGE_R * S * 1.3 * k.powf(0.6);
            draw_rectangle(g.x - w, top, w * 2.0, g.y - top, alpha(GOLD, 0.25 * k));
            draw_rectangle(g.x - w * 0.6, top, w * 1.2, g.y - top, alpha(PALE, 0.6 * k));
            draw_rectangle(g.x - w * 0.25, top, w * 0.5, g.y - top, alpha(WHITE, k));
            glow(g, w * 0.8, GOLD, k);
        }
        for r in &self.rings {
            if r.z <= 0.0 {
                continue;
            }
            let k = r.t / r.max;
            let c = air(r.pos, r.z);
            let rr = (r.r1 + (r.r0 - r.r1) * k) * S;
            draw_circle_lines(c.x, c.y, rr, r.w * (0.5 + k), alpha(r.col, k));
        }
        for n in &self.nums {
            let k = n.t / 0.8;
            let c = air(n.pos, 16.0 + (1.0 - k) * 12.0);
            let size = if n.dmg >= 20 { 40.0 } else { 26.0 } * (1.0 + 0.4 * ((1.0 - k) * 8.0).min(1.0) * k);
            let txt = n.dmg.to_string();
            let col = if n.dmg >= 30 { GOLD } else if n.dmg >= 20 { PALE } else { WHITE };
            draw_text(&txt, c.x - size * 0.25 + 2.0, c.y + 2.0, size, alpha(BLACK, k));
            draw_text(&txt, c.x - size * 0.25, c.y, size, alpha(col, k));
        }
    }

    pub fn draw_flash(&self) {
        if self.flash > 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), alpha(Color::new(1.0, 0.97, 0.9, 1.0), self.flash * 1.5));
        }
    }
}
