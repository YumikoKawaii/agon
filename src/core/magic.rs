use macroquad::math::Vec2;

use crate::core::{
    battle::{bless, strike, Hit},
    effect::{Apply, EffectKind},
    event::{Element, Event},
    inside,
    player::Player,
    seg_dist, Skill, ARENA, DT, RADIUS,
};

pub const NEEDLE_SPEED: f32 = 320.0;
pub const NEEDLE_R: f32 = 2.5;
const NEEDLE_DMG: i32 = 1;
const NEEDLE_LIFE: f32 = 0.8;
const TIP: f32 = 8.0;

pub const RAY_LEN: f32 = 280.0;
pub const RAY_CHARGE: f32 = 0.4;
pub const RAY_SHOW: f32 = 0.3;
pub const RAY_W: f32 = 7.0;
const RAY_DMG: i32 = 11;
const NEEDLE_FX: &[Apply] = &[Apply(EffectKind::Slow, 30, 15)];
const RAY_FX: &[Apply] = &[Apply(EffectKind::Slow, 72, 40)];
const HEAVEN_FX: &[Apply] = &[Apply(EffectKind::Slow, 60, 40)];
const WARD_SHIELD: Apply = Apply(EffectKind::Shield, 120, 8);

pub const WARD_T: f32 = 1.5;
pub const WARD_W: f32 = 22.0;
pub const WARD_OFF: f32 = 14.0;
const WARD_REFUND: f32 = 2.0;

pub const JUDGE_RANGE: f32 = RAY_LEN;
pub const JUDGE_R: f32 = 42.0;
pub const JUDGE_T: f32 = 0.7;
pub const HEAVEN_N: usize = 12;
pub const HEAVEN_RING: f32 = 26.0;
pub const HEAVEN_H: f32 = 46.0;
pub const HEAVEN_SHOW: f32 = 0.45;
const CORE_R: f32 = 14.0;
const CORE_DMG: i32 = 20;
const EDGE_DMG: i32 = 7;

pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: usize,
    life: f32,
}

pub struct Ray {
    pub origin: Vec2,
    pub dir: Vec2,
    pub t: f32,
    pub owner: usize,
    pub fired: bool,
}

impl Ray {
    pub fn tip(&self) -> Vec2 {
        self.origin + self.dir * TIP
    }

    pub fn end(&self) -> Vec2 {
        self.origin + self.dir * RAY_LEN
    }
}

pub struct Ward {
    pub owner: usize,
    pub t: f32,
}

pub struct Judge {
    pub pos: Vec2,
    pub t: f32,
    pub owner: usize,
    pub from: Vec2,
    pub fired: bool,
}

impl Judge {
    pub fn charge(&self) -> f32 {
        (self.t / JUDGE_T).min(1.0)
    }

    pub fn emitter(&self, n: usize) -> Vec2 {
        let base = (self.pos - self.from).normalize_or(Vec2::X);
        let ang = base.y.atan2(base.x) + n as f32 * std::f32::consts::TAU / HEAVEN_N as f32;
        self.from + Vec2::from_angle(ang) * HEAVEN_RING
    }
}

pub fn ward_segment(p: &Player) -> (Vec2, Vec2) {
    let f = p.facing();
    let c = p.pos + f * WARD_OFF;
    (c - f.perp() * WARD_W, c + f.perp() * WARD_W)
}

pub fn ward_block(wards: &[Ward], players: &mut [Player; 2], owner: usize, pos: Vec2, r: f32, events: &mut Vec<Event>) -> bool {
    let Some(o) = wards.iter().map(|w| w.owner).filter(|&o| o != owner).find(|&o| {
        let (a, b) = ward_segment(&players[o]);
        seg_dist(pos, a, b) < r + 3.0
    }) else {
        return false;
    };
    if let Some(slot) = players[o].kit().skills.iter().position(|k| *k == Skill::Ward) {
        players[o].cd[slot] = (players[o].cd[slot] - WARD_REFUND).max(0.0);
    }
    events.push(Event::Blocked { pos });
    players[o].haste = crate::core::player::GRACE_T;
    bless(players, o, WARD_SHIELD, events);
    true
}

pub fn judge_target(p: &Player) -> Vec2 {
    inside(p.pos + (p.target - p.pos).clamp_length_max(JUDGE_RANGE), JUDGE_R * 0.6)
}

#[derive(Default)]
pub struct Spells {
    pub shots: Vec<Shot>,
    pub rays: Vec<Ray>,
    pub wards: Vec<Ward>,
    pub judges: Vec<Judge>,
}

impl Spells {
    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let p = &players[i];
        let f = p.facing();
        match skill {
            Skill::Needle => {
                let tip = p.pos + f * TIP;
                self.shots.push(Shot { pos: tip, vel: f * NEEDLE_SPEED, owner: i, life: NEEDLE_LIFE });
                events.push(Event::Needle { tip, ang: p.ang });
            }
            Skill::Ray => self.rays.push(Ray { origin: p.pos, dir: f, t: 0.0, owner: i, fired: false }),
            Skill::Ward => {
                self.wards.push(Ward { owner: i, t: 0.0 });
                events.push(Event::Ward { pos: p.pos, front: p.pos + f * WARD_OFF });
            }
            Skill::Judgement => {
                let at = judge_target(p);
                self.judges.push(Judge { pos: at, t: 0.0, owner: i, from: p.pos, fired: false });
                events.push(Event::JudgeCast { caster: p.pos, at });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        self.update_shots(players, events);
        self.update_rays(players, events);
        self.wards.retain_mut(|w| {
            w.t += DT;
            w.t < WARD_T
        });
        self.update_judges(players, events);
    }

    fn update_shots(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        let wards = &self.wards;
        self.shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            if ward_block(wards, players, s.owner, s.pos, NEEDLE_R, events) {
                return false;
            }
            let t = 1 - s.owner;
            let hit = players[t].hurtbox().is_some_and(|r| players[t].pos.distance(s.pos) < r + NEEDLE_R);
            if hit {
                strike(players, t, Hit::new(NEEDLE_DMG, s.vel.normalize_or_zero(), Element::Frost, false, NEEDLE_FX), events);
            }
            let alive = s.pos.cmpge(Vec2::ZERO).all() && s.pos.cmple(ARENA).all();
            !hit && s.life > 0.0 && alive
        });
    }

    fn update_rays(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        self.rays.retain_mut(|r| {
            r.t += DT;
            if !r.fired {
                r.origin = players[r.owner].pos;
            }
            if r.t >= RAY_CHARGE && !r.fired {
                r.fired = true;
                let t = 1 - r.owner;
                if seg_dist(players[t].pos, r.origin, r.end()) < RAY_W + RADIUS {
                    strike(players, t, Hit::new(RAY_DMG, r.dir * 2.5 / 3.0, Element::Frost, true, RAY_FX), events);
                }
                events.push(Event::RayFired { origin: r.origin, dir: r.dir });
            }
            r.t < RAY_CHARGE + RAY_SHOW
        });
    }

    fn update_judges(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        self.judges.retain_mut(|j| {
            j.t += DT;
            if j.t < JUDGE_T {
                j.from = players[j.owner].pos;
            }
            if !j.fired && j.t >= JUDGE_T {
                j.fired = true;
                let t = 1 - j.owner;
                let d = players[t].pos - j.pos;
                let dist = d.length();
                if dist < JUDGE_R + RADIUS {
                    let core = dist < CORE_R + RADIUS;
                    let dmg = if core { CORE_DMG } else { EDGE_DMG };
                    strike(players, t, Hit::new(dmg, d.normalize_or(Vec2::X) * if core { 4.0 } else { 2.0 }, Element::Holy, core, HEAVEN_FX), events);
                }
                events.push(Event::JudgeStrike { pos: j.pos });
            }
            j.t < JUDGE_T + HEAVEN_SHOW
        });
    }
}
