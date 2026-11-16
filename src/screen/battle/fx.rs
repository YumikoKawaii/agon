use std::f32::consts::TAU;

use macroquad::{prelude::*, rand::gen_range};

use crate::{
    assets,
    core::{
        anim::State,
        effect::EffectKind,
        event::{Element, Event},
        magic::{ward_segment, JUDGE_R, NEEDLE_R},
        hex::{CAULDRON_R, POTION_R},
        player::dir8,
        sword::{FORM_N, FORM_R, INTENT_R, LUNGE_REACH},
        Battle, Player,
    },
    screen::battle::view::{alpha, draw_sprite, glow, status_color, Camera, AQUA, BLOOD, CHEST, GOLD, ICE, LILAC, PALE, S, VIEW},
};

const SHAKE_DECAY: f32 = 0.86;
const NUM_LIFE: f32 = 0.8;
const PILLAR_LIFE: f32 = 0.5;
const ARC_LIFE: f32 = 0.16;
const SLASH_LIFE: f32 = 0.25;
const GHOST_TINT: Color = Color::new(0.45, 1.0, 0.92, 1.0);

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

struct Arc {
    owner: usize,
    ang: f32,
    r: f32,
    half: f32,
    side: f32,
    t: f32,
    big: bool,
}

struct Slash {
    a: Vec2,
    b: Vec2,
    t: f32,
}

struct Ghost {
    seat: usize,
    pos: Vec2,
    ang: f32,
    t: f32,
}

struct Num {
    pos: Vec2,
    t: f32,
    dmg: i32,
    tag: Option<Color>,
}

struct Pillar {
    pos: Vec2,
    t: f32,
}

fn element_color(e: Element) -> Color {
    match e {
        Element::Frost => ICE,
        Element::Holy => GOLD,
        Element::Blade => AQUA,
        Element::Hex => LILAC,
    }
}

fn pick(a: Color, b: Color) -> Color {
    if gen_range(0, 2) == 0 { a } else { b }
}

#[derive(Clone, Copy)]
enum Kind {
    Potion,
    Burst,
    Poof,
    BleedPop,
    Release,
}

struct Sprite {
    kind: Kind,
    pos: Vec2,
    t: f32,
}

fn sheet(kind: Kind) -> (&'static assets::Sheet, f32) {
    let w = &assets::get().witch;
    match kind {
        Kind::Potion => (&w.potion, 0.7),
        Kind::Burst => (&w.burst, 0.85),
        Kind::Poof => (&w.poof, 0.6),
        Kind::BleedPop => (&assets::get().bleed_pop.sheet, 0.6),
        Kind::Release => (&assets::get().sword.release, 0.6),
    }
}

#[derive(Default)]
pub struct Fx {
    sprites: Vec<Sprite>,
    parts: Vec<Part>,
    rings: Vec<Ring>,
    arcs: Vec<Arc>,
    slashes: Vec<Slash>,
    ghosts: Vec<Ghost>,
    nums: Vec<Num>,
    pillars: Vec<Pillar>,
    pub shake: f32,
    flash: f32,
}

impl Fx {
    #[allow(clippy::too_many_arguments)]
    fn part(&mut self, pos: Vec2, z: f32, vel: Vec2, vz: f32, life: f32, size: f32, col: Color, drag: f32) {
        self.parts.push(Part { pos, z, vel, vz, life, max: life, size, col, drag });
    }

    fn burst(&mut self, pos: Vec2, z: f32, n: usize, speed: f32, life: f32, size: f32, col: Color) {
        for _ in 0..n {
            let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(speed * 0.3, speed);
            self.part(pos, z, v, gen_range(-speed * 0.3, speed * 0.6), gen_range(life * 0.5, life), gen_range(size * 0.5, size), col, 0.9);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn ring(&mut self, pos: Vec2, z: f32, r0: f32, r1: f32, max: f32, col: Color, w: f32) {
        self.rings.push(Ring { pos, z, r0, r1, t: max, max, col, w });
    }

    fn kick(&mut self, shake: f32, flash: f32) {
        self.shake = self.shake.max(shake);
        self.flash = self.flash.max(flash);
    }

    pub fn spawn(&mut self, event: &Event, players: &[Player; 2]) {
        match *event {
            Event::Needle { tip, ang } => {
                for _ in 0..6 {
                    let v = Vec2::from_angle(ang + gen_range(-0.6, 0.6)) * gen_range(40.0, 110.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 20.0), 0.25, 1.5, PALE, 0.85);
                }
                self.ring(tip, CHEST, 1.0, 7.0, 0.15, ICE, 2.0);
            }
            Event::Ward { pos, front } => {
                self.ring(pos, CHEST, 4.0, 30.0, 0.35, PALE, 3.0);
                self.burst(front, CHEST, 16, 60.0, 0.4, 2.0, ICE);
            }
            Event::Blocked { pos } => {
                self.burst(pos, CHEST, 18, 120.0, 0.35, 2.0, PALE);
                self.ring(pos, CHEST, 2.0, 16.0, 0.25, ICE, 3.0);
                self.kick(2.5, 0.0);
            }
            Event::RayFired { origin, dir } => {
                for _ in 0..50 {
                    let along = origin + dir * gen_range(10.0, crate::core::magic::RAY_LEN);
                    let v = dir.perp() * gen_range(-90.0, 90.0) + dir * gen_range(20.0, 120.0);
                    let col = if gen_range(0, 3) == 0 { WHITE } else { ICE };
                    self.part(along, CHEST, v, gen_range(-20.0, 40.0), gen_range(0.2, 0.5), gen_range(1.0, 2.5), col, 0.88);
                }
                self.ring(origin + dir * 8.0, CHEST, 3.0, 22.0, 0.25, PALE, 4.0);
                self.kick(6.0, 0.12);
            }
            Event::JudgeCast { caster, at } => {
                self.ring(at, 0.0, JUDGE_R * 1.6, JUDGE_R, 0.4, GOLD, 3.0);
                self.burst(caster, CHEST + 8.0, 20, 70.0, 0.5, 2.0, GOLD);
            }
            Event::JudgeStrike { pos } => {
                self.pillars.push(Pillar { pos, t: PILLAR_LIFE });
                self.ring(pos, 0.0, 4.0, JUDGE_R * 1.8, 0.5, GOLD, 6.0);
                self.ring(pos, 0.0, 4.0, JUDGE_R * 1.3, 0.35, WHITE, 4.0);
                self.ring(pos, CHEST, 4.0, JUDGE_R, 0.25, PALE, 3.0);
                for _ in 0..90 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(60.0, 220.0);
                    let col = if gen_range(0, 3) == 0 { WHITE } else { GOLD };
                    self.part(pos, gen_range(0.0, 10.0), v, gen_range(20.0, 200.0), gen_range(0.4, 0.9), gen_range(1.5, 3.5), col, 0.92);
                }
                self.kick(14.0, 0.35);
            }
            Event::Cut { owner, ang, r, half, side, big } => {
                self.arcs.push(Arc { owner, ang, r, half, side, t: ARC_LIFE, big });
                let pos = players[owner].pos;
                for _ in 0..(if big { 14 } else { 6 }) {
                    let a = ang + gen_range(-half, half);
                    self.part(pos + Vec2::from_angle(a) * r * 0.8, CHEST, Vec2::from_angle(a) * gen_range(30.0, 90.0), gen_range(0.0, 20.0), 0.25, 1.5, pick(AQUA, WHITE), 0.88);
                }
                if big {
                    self.kick(3.0, 0.0);
                }
            }
            Event::Swallow { pos, dir } => {
                self.ring(pos, CHEST, 2.0, 14.0, 0.2, AQUA, 3.0);
                for _ in 0..10 {
                    let v = -dir * gen_range(40.0, 120.0) + dir.perp() * gen_range(-50.0, 50.0);
                    self.part(pos, CHEST, v, gen_range(0.0, 20.0), 0.3, 1.6, PALE, 0.88);
                }
            }
            Event::Intent { pos, dir } => {
                self.sprites.push(Sprite { kind: Kind::Release, pos: pos + dir * 8.0, t: 0.0 });
                self.ring(pos, CHEST, 4.0, 26.0, 0.3, AQUA, 4.0);
                for _ in 0..24 {
                    let v = dir * gen_range(60.0, 200.0) + dir.perp() * gen_range(-80.0, 80.0);
                    self.part(pos, CHEST, v, gen_range(0.0, 30.0), 0.35, 1.8, pick(AQUA, LILAC), 0.88);
                }
                self.kick(5.0, 0.06);
            }
            Event::IntentFade { pos, dir } => {
                for _ in 0..16 {
                    let a = gen_range(-1.0, 1.0);
                    let p = pos + dir.perp() * a * INTENT_R;
                    self.part(p, CHEST, dir * gen_range(20.0, 70.0), gen_range(0.0, 20.0), 0.35, 1.6, pick(AQUA, PALE), 0.9);
                }
            }
            Event::SwallowHit { tip } => {
                self.ring(tip, CHEST, 2.0, 20.0, 0.3, WHITE, 4.0);
                self.burst(tip, CHEST, 16, 140.0, 0.35, 2.0, AQUA);
            }
            Event::Wind { from, to, dir } => {
                self.slashes.push(Slash { a: from, b: to, t: SLASH_LIFE });
                for _ in 0..16 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-6.0, 6.0), dir.perp() * gen_range(-60.0, 60.0) - dir * 30.0, gen_range(0.0, 30.0), 0.35, 1.6, pick(AQUA, PALE), 0.88);
                }
            }
            Event::Ghost { seat, pos, ang, life } => self.ghosts.push(Ghost { seat, pos, ang, t: life }),
            Event::FormCast { caster, center } => {
                self.ring(center, 0.0, FORM_R * 2.0, FORM_R, 0.4, AQUA, 3.0);
                self.burst(caster, CHEST + 6.0, 24, 90.0, 0.4, 2.0, AQUA);
            }
            Event::FormBurst { center, base } => {
                self.ring(center, 0.0, 4.0, FORM_R * 2.2, 0.45, AQUA, 6.0);
                self.ring(center, CHEST, 4.0, FORM_R * 1.4, 0.3, WHITE, 4.0);
                for n in 0..FORM_N {
                    let d = Vec2::from_angle(base + n as f32 * TAU / FORM_N as f32);
                    self.burst(center - d * FORM_R, CHEST, 6, 90.0, 0.4, 2.0, PALE);
                }
                self.burst(center, CHEST, 50, 180.0, 0.6, 2.6, AQUA);
                self.kick(9.0, 0.18);
            }
            Event::HexCast { tip } => {
                self.burst(tip, CHEST, 8, 50.0, 0.3, 1.8, LILAC);
            }
            Event::PotionBurst { pos } => {
                self.sprites.push(Sprite { kind: Kind::Potion, pos, t: 0.0 });
                self.ring(pos, 0.0, 4.0, POTION_R * 1.6, 0.4, LILAC, 5.0);
                self.burst(pos, 6.0, 40, 150.0, 0.5, 2.4, LILAC);
                self.burst(pos, 6.0, 14, 90.0, 0.4, 1.6, WHITE);
                self.kick(5.0, 0.0);
            }
            Event::Broom { from, to, dir } => {
                self.sprites.push(Sprite { kind: Kind::Poof, pos: from, t: 0.0 });
                self.sprites.push(Sprite { kind: Kind::Poof, pos: to, t: 0.0 });
                for _ in 0..18 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-6.0, 6.0), dir.perp() * gen_range(-50.0, 50.0) - dir * 30.0, gen_range(0.0, 30.0), 0.4, 1.8, pick(LILAC, GOLD), 0.88);
                }
            }
            Event::CauldronCast { caster, at } => {
                self.ring(at, 0.0, CAULDRON_R * 1.6, CAULDRON_R, 0.4, LILAC, 3.0);
                self.burst(caster, CHEST + 8.0, 20, 70.0, 0.5, 2.0, LILAC);
            }
            Event::CauldronBurst { pos } => {
                self.sprites.push(Sprite { kind: Kind::Burst, pos, t: 0.0 });
                self.ring(pos, 0.0, 4.0, CAULDRON_R * 1.8, 0.5, LILAC, 6.0);
                self.ring(pos, 0.0, 4.0, CAULDRON_R * 1.2, 0.35, WHITE, 4.0);
                for _ in 0..80 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(50.0, 200.0);
                    self.part(pos, gen_range(0.0, 8.0), v, gen_range(40.0, 220.0), gen_range(0.4, 0.9), gen_range(1.5, 3.5), pick(LILAC, AQUA), 0.92);
                }
                self.kick(12.0, 0.25);
            }
            Event::Hit { pos, element, dmg, big } => {
                let col = element_color(element);
                let n = if big { 30 } else { 10 };
                self.burst(pos, CHEST, n, if big { 140.0 } else { 80.0 }, 0.45, if big { 3.0 } else { 2.0 }, col);
                self.burst(pos, CHEST, n / 2, 60.0, 0.3, 1.5, WHITE);
                self.ring(pos, CHEST, 2.0, if big { 26.0 } else { 12.0 }, 0.3, col, 3.0);
                self.nums.push(Num { pos: pos + vec2(gen_range(-4.0, 4.0), 0.0), t: NUM_LIFE, dmg, tag: None });
                self.kick(if big { 7.0 } else { 3.0 }, 0.0);
            }
            Event::Status { pos, kind } => {
                let col = status_color(kind);
                self.ring(pos, CHEST, 3.0, 20.0, 0.35, col, 3.0);
                self.burst(pos, CHEST, 14, 90.0, 0.5, 2.2, col);
                for n in 0..10 {
                    let a = n as f32 / 10.0 * TAU;
                    self.part(pos + Vec2::from_angle(a) * 12.0, 0.0, Vec2::from_angle(a) * -20.0, gen_range(60.0, 120.0), 0.5, 1.6, pick(col, WHITE), 0.92);
                }
                if kind == EffectKind::Stun || kind == EffectKind::Root {
                    self.kick(6.0, 0.08);
                }
            }
            Event::Dot { pos, kind, dmg } => {
                let col = status_color(kind);
                self.burst(pos, CHEST, 6, 40.0, 0.35, 1.6, col);
                self.nums.push(Num { pos: pos + vec2(gen_range(-6.0, 6.0), 0.0), t: NUM_LIFE, dmg, tag: Some(col) });
            }
            Event::Absorb { pos, amount } => {
                self.ring(pos, CHEST, 6.0, 22.0, 0.3, GOLD, 4.0);
                self.burst(pos, CHEST, 18, 120.0, 0.4, 2.0, pick(GOLD, PALE));
                self.nums.push(Num { pos: pos + vec2(0.0, -4.0), t: NUM_LIFE, dmg: -amount, tag: Some(GOLD) });
            }
            Event::MarkPop { pos } => {
                self.ring(pos, CHEST, 2.0, 34.0, 0.4, GOLD, 5.0);
                self.ring(pos, 0.0, 4.0, 30.0, 0.5, PALE, 3.0);
                self.burst(pos, CHEST, 30, 180.0, 0.6, 2.6, pick(GOLD, WHITE));
                self.kick(10.0, 0.2);
            }
            Event::BleedPop { pos, dmg } => {
                self.sprites.push(Sprite { kind: Kind::BleedPop, pos, t: 0.0 });
                self.burst(pos, CHEST, 20 + dmg as usize * 2, 160.0, 0.6, 2.4, pick(BLOOD, AQUA));
                self.ring(pos, CHEST, 2.0, 28.0, 0.3, BLOOD, 4.0);
                self.kick(9.0, 0.12);
            }
        }
    }

    pub fn ambient(&mut self, b: &Battle) {
        for s in &b.spells.shots {
            let (pos, vel) = (s.pos, s.vel);
            self.part(pos, CHEST + gen_range(-1.0, 1.0), -vel * 0.1, 0.0, 0.25, NEEDLE_R * 0.7, ICE, 0.9);
        }
        for r in b.spells.rays.iter().filter(|r| !r.fired) {
            let tip = r.tip();
            for _ in 0..3 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(14.0, 26.0);
                self.part(tip + off, CHEST + gen_range(-8.0, 8.0), -off * 4.0, 0.0, 0.22, 1.8, pick(ICE, PALE), 1.0);
            }
        }
        for w in &b.spells.wards {
            if gen_range(0, 3) == 0 {
                let (a, c) = ward_segment(&b.players[w.owner]);
                self.part(a.lerp(c, gen_range(0.0, 1.0)), CHEST + gen_range(-6.0, 6.0), Vec2::ZERO, 18.0, 0.4, 1.3, PALE, 1.0);
            }
        }
        for j in &b.spells.judges {
            for _ in 0..(1 + (j.charge() * 4.0) as usize) {
                let p = j.pos + Vec2::from_angle(gen_range(0.0, TAU)) * JUDGE_R * gen_range(0.85, 1.0);
                self.part(p, 0.0, Vec2::ZERO, gen_range(40.0, 120.0), 0.5, gen_range(1.0, 2.2), pick(GOLD, PALE), 1.0);
            }
        }
        for w in &b.blades.waves {
            for _ in 0..3 {
                let p = w.pos + w.dir.perp() * gen_range(-1.0, 1.0) * INTENT_R * 0.9;
                self.part(p, CHEST + gen_range(-4.0, 4.0), -w.dir * gen_range(30.0, 90.0), 0.0, 0.3, 1.6, pick(AQUA, LILAC), 0.9);
            }
        }
        for p in b.players.iter().filter(|p| p.intent_ready()) {
            if gen_range(0, 2) == 0 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(6.0, 12.0);
                self.part(p.pos + off, gen_range(0.0, CHEST), -off * 1.5, gen_range(30.0, 70.0), 0.45, gen_range(1.0, 2.0), pick(AQUA, LILAC), 0.95);
            }
        }
        for l in &b.blades.lunges {
            let tip = b.players[l.owner].pos + l.dir * LUNGE_REACH;
            self.part(tip, CHEST, -l.dir * 60.0 + l.dir.perp() * gen_range(-30.0, 30.0), 0.0, 0.25, 1.5, pick(AQUA, WHITE), 0.88);
        }
        for fm in &b.blades.forms {
            for n in 0..FORM_N {
                let s = fm.stab(n);
                if !(0.0..=1.0).contains(&s) {
                    continue;
                }
                self.part(fm.sword_pos(n), CHEST, -fm.dir(n) * 40.0, 0.0, 0.2, 1.4, AQUA, 0.9);
                if s > 0.45 && s < 0.55 {
                    self.ring(fm.center, CHEST, 1.0, 8.0, 0.15, AQUA, 2.0);
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.sprites.retain_mut(|s| {
            s.t += dt;
            sheet(s.kind).0.frame_at(s.t, false).is_some()
        });
        self.parts.retain_mut(|p| {
            p.pos += p.vel * dt;
            p.z = (p.z + p.vz * dt).max(0.0);
            p.vel *= p.drag;
            p.vz *= p.drag;
            p.life -= dt;
            p.life > 0.0
        });
        self.rings.retain_mut(|r| {
            r.t -= dt;
            r.t > 0.0
        });
        self.arcs.retain_mut(|a| {
            a.t -= dt;
            a.t > 0.0
        });
        self.slashes.retain_mut(|s| {
            s.t -= dt;
            s.t > 0.0
        });
        self.ghosts.retain_mut(|g| {
            g.t -= dt;
            g.t > 0.0
        });
        self.nums.retain_mut(|n| {
            n.t -= dt;
            n.t > 0.0
        });
        self.pillars.retain_mut(|p| {
            p.t -= dt;
            p.t > 0.0
        });
        self.shake *= SHAKE_DECAY;
        self.flash = (self.flash - dt).max(0.0);
    }

    fn draw_ring(cam: &Camera, r: &Ring) {
        let k = r.t / r.max;
        let c = cam.air(r.pos, r.z);
        let rr = (r.r1 + (r.r0 - r.r1) * k) * S;
        draw_circle_lines(c.x, c.y, rr, r.w * (0.5 + k), alpha(r.col, k));
    }

    pub fn draw_ground(&self, cam: &Camera) {
        self.rings.iter().filter(|r| r.z <= 0.0).for_each(|r| Self::draw_ring(cam, r));
    }

    pub fn draw_ghosts(&self, cam: &Camera, seat: usize, kit: usize) {
        for g in self.ghosts.iter().filter(|g| g.seat == seat) {
            let tex = assets::skin(kit).frame(State::Run, dir8(g.ang), 0, 1);
            draw_sprite(tex, cam.proj(g.pos), alpha(GHOST_TINT, (g.t / 0.3).min(1.0) * 0.55));
        }
    }

    pub fn draw_air(&self, cam: &Camera, players: &[Player; 2]) {
        for s in &self.sprites {
            let (sh, k) = sheet(s.kind);
            if let Some(i) = sh.frame_at(s.t, false) {
                sh.draw(i, cam.air(s.pos, 8.0), k * S, 1.0);
            }
        }
        for p in &self.parts {
            let k = p.life / p.max;
            let c = cam.air(p.pos, p.z);
            draw_circle(c.x, c.y, p.size * S * 1.8, alpha(p.col, k * 0.2));
            draw_circle(c.x, c.y, p.size * S * (0.5 + 0.5 * k), alpha(p.col, k));
        }
        for p in &self.pillars {
            let k = p.t / PILLAR_LIFE;
            let g = cam.proj(p.pos);
            let top = g.y - 900.0;
            let w = JUDGE_R * S * 1.3 * k.powf(0.6);
            draw_rectangle(g.x - w, top, w * 2.0, g.y - top, alpha(GOLD, 0.25 * k));
            draw_rectangle(g.x - w * 0.6, top, w * 1.2, g.y - top, alpha(PALE, 0.6 * k));
            draw_rectangle(g.x - w * 0.25, top, w * 0.5, g.y - top, alpha(WHITE, k));
            glow(g, w * 0.8, GOLD, k);
        }
        self.rings.iter().filter(|r| r.z > 0.0).for_each(|r| Self::draw_ring(cam, r));
        for a in &self.arcs {
            let k = a.t / ARC_LIFE;
            let pos = players[a.owner].pos;
            let segs = 14;
            let spread = a.half * 2.0 * (0.4 + 0.6 * (1.0 - k) * 1.6).min(1.0);
            let start = a.ang - a.side * a.half;
            let pts: Vec<Vec2> = (0..=segs).map(|n| cam.air(pos + Vec2::from_angle(start + a.side * spread * n as f32 / segs as f32) * a.r, CHEST)).collect();
            let w = if a.big { 9.0 } else { 6.0 };
            for n in 0..segs {
                let taper = n as f32 / segs as f32;
                let (x, y) = (pts[n], pts[n + 1]);
                draw_line(x.x, x.y, y.x, y.y, w * 2.2 * taper * k, alpha(AQUA, 0.25 * k));
                draw_line(x.x, x.y, y.x, y.y, w * taper * k, alpha(AQUA, 0.8 * k));
                draw_line(x.x, x.y, y.x, y.y, w * 0.4 * taper * k, alpha(WHITE, k));
            }
            glow(pts[segs], w * 0.6 * k, AQUA, k);
        }
        for s in &self.slashes {
            let k = s.t / SLASH_LIFE;
            let (a, b) = (cam.air(s.a, CHEST), cam.air(s.b, CHEST));
            let w = 3.0 * S * k;
            draw_line(a.x, a.y, b.x, b.y, w * 2.4, alpha(AQUA, 0.2 * k));
            draw_line(a.x, a.y, b.x, b.y, w, alpha(AQUA, 0.75 * k));
            draw_line(a.x, a.y, b.x, b.y, w * 0.35, alpha(WHITE, k));
        }
    }

    pub fn draw_numbers(&self, cam: &Camera) {
        for n in &self.nums {
            let k = n.t / NUM_LIFE;
            let c = cam.air(n.pos, 16.0 + (1.0 - k) * 12.0);
            let base = if n.tag.is_some() { 20.0 } else if n.dmg >= 20 { 40.0 } else { 26.0 };
            let size = base * (1.0 + 0.4 * ((1.0 - k) * 8.0).min(1.0) * k);
            let txt = if n.dmg < 0 { format!("({})", -n.dmg) } else { n.dmg.to_string() };
            let col = n.tag.unwrap_or(if n.dmg >= 30 { GOLD } else if n.dmg >= 20 { PALE } else { WHITE });
            draw_text(&txt, c.x - size * 0.25 + 2.0, c.y + 2.0, size, alpha(BLACK, k));
            draw_text(&txt, c.x - size * 0.25, c.y, size, alpha(col, k));
        }
    }

    pub fn draw_flash(&self) {
        if self.flash > 0.0 {
            draw_rectangle(0.0, 0.0, VIEW.x, VIEW.y, alpha(Color::new(1.0, 0.97, 0.9, 1.0), self.flash * 1.5));
        }
    }
}
