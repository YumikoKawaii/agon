use macroquad::math::Vec2;

use crate::core::{
    battle::{strike, Hit},
    effect::{Apply, EffectKind},
    event::{Element, Event},
    inside,
    player::Player,
    seg_dist, Skill, ARENA, DT, RADIUS,
};

pub const SLASH_R: f32 = 24.0;
pub const SLASH_HALF: f32 = 1.2;
const SLASH_DMG: i32 = 4;
const SLASH_FIN_DMG: i32 = 6;
const BLEED: &[Apply] = &[Apply(EffectKind::Bleed, 180, 1)];

pub const WHIRL_R: f32 = 28.0;
pub const WHIRL_TICKS: u32 = 18;
const WHIRL_HITS: [u32; 3] = [2, 8, 14];
const WHIRL_DMG: i32 = 4;
const WHIRL_DRIFT: f32 = 30.0;

pub const CABLE_RANGE: f32 = 180.0;
pub const HOOK_SPEED: f32 = 560.0;
const PULL_SPEED: f32 = 340.0;
pub const GAS_MAX: f32 = 100.0;
pub const GAS_COST: f32 = 25.0;
pub const GAS_REGEN: f32 = 20.0;
pub const CABLE_GAP: f32 = 0.0;
const ZIP_DMG: i32 = 5;

pub const FURY_T: f32 = 4.0;
const FURY_HEAL: i32 = 2;

pub struct Hook {
    pub owner: usize,
    pub pos: Vec2,
    pub to: Vec2,
    pub pulling: bool,
    start: Vec2,
    passed: bool,
    attach: bool,
    back: bool,
}

pub const CHAIN_MAX: u32 = 3;
pub const CHAIN_WINDOW: f32 = 2.0;
const CHAIN_DISCOUNT: f32 = 5.0;
const GAS_FLOOR: f32 = 10.0;

pub fn cable_cost(p: &Player) -> f32 {
    (GAS_COST - CHAIN_DISCOUNT * p.chain as f32).max(GAS_FLOOR)
}
const CHAIN_SPEED: f32 = 0.25;
const CHAIN_DMG: i32 = 2;

fn wall_dist(p: Vec2, d: Vec2) -> f32 {
    let m = RADIUS + 2.0;
    let tx = if d.x > 0.0 { (ARENA.x - m - p.x) / d.x } else if d.x < 0.0 { (m - p.x) / d.x } else { f32::MAX };
    let ty = if d.y > 0.0 { (ARENA.y - m - p.y) / d.y } else if d.y < 0.0 { (m - p.y) / d.y } else { f32::MAX };
    tx.min(ty).max(0.0)
}

pub struct Whirl {
    pub owner: usize,
    pub tick: u32,
    pub dir: Vec2,
    pub big: bool,
    hit: [bool; 3],
}

fn power(p: &Player, dmg: i32) -> i32 {
    if p.fury > 0.0 && p.hp * 2 < p.max_hp() {
        dmg * 3 / 2
    } else {
        dmg
    }
}

#[derive(Default)]
pub struct Wardens {
    pub hooks: Vec<Hook>,
    pub whirls: Vec<Whirl>,
}

impl Wardens {
    fn fire(&mut self, players: &mut [Player; 2], i: usize, events: &mut Vec<Event>) {
        let chained = players[i].chain_t > 0.0;
        self.hooks.retain(|h| h.owner != i);
        players[i].chain = if chained { (players[i].chain + 1).min(CHAIN_MAX) } else { 0 };
        players[i].chain_t = CHAIN_WINDOW;
        let pos = players[i].pos;
        let dir = (players[i].target - pos).normalize_or(players[i].facing());
        let attach = true;
        let to = inside(pos + dir * wall_dist(pos, dir), RADIUS + 2.0);
        self.hooks.push(Hook { owner: i, pos, to, pulling: false, start: pos, passed: false, attach, back: false });
        events.push(Event::HookFire { from: pos, to });
    }

    fn whirl(&mut self, players: &[Player; 2], i: usize, big: bool, events: &mut Vec<Event>) {
        self.whirls.retain(|w| w.owner != i);
        let dir = (players[i].target - players[i].pos).normalize_or_zero();
        self.whirls.push(Whirl { owner: i, tick: 0, dir, big, hit: [false; 3] });
        events.push(Event::Whirl { owner: i, big });
    }

    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let t = 1 - i;
        let f = players[i].facing();
        let pos = players[i].pos;
        match skill {
            Skill::Slash => {
                let big = players[i].combo == 2;
                let side = if big { 1.0 } else { -1.0 };
                events.push(Event::Twin { owner: i, ang: players[i].ang, side, big });
                let to = players[t].pos - pos;
                if to.length() < SLASH_R + RADIUS && f.angle_between(to).abs() < SLASH_HALF {
                    let (dmg, apply) = if big { (SLASH_FIN_DMG, BLEED) } else { (SLASH_DMG, &[][..]) };
                    strike(players, t, Hit::new(power(&players[i], dmg), f, Element::Blade, big, apply), events);
                }
            }
            Skill::Whirl => self.whirl(players, i, false, events),
            Skill::Cable => self.fire(players, i, events),
            Skill::Fury => {
                players[i].fury = FURY_T;
                players[i].gas = GAS_MAX;
                events.push(Event::Fury { pos });
                self.fire(players, i, events);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        let mut spins = Vec::new();
        self.hooks.retain_mut(|h| {
            let o = h.owner;
            if players[o].state == crate::core::anim::State::Hit {
                players[o].flying = false;
                players[o].chain = 0;
                return false;
            }
            if h.back {
                let home = players[o].pos;
                let left = home - h.pos;
                let step = HOOK_SPEED * 1.4 * DT;
                if left.length() <= step {
                    return false;
                }
                h.pos += left.normalize() * step;
                return true;
            }
            if !h.pulling {
                let step = HOOK_SPEED * DT;
                let left = h.to - h.pos;
                if left.length() <= step {
                    h.pos = h.to;
                    let cost = cable_cost(&players[o]);
                    let paid = players[o].fury > 0.0 || players[o].gas >= cost;
                    if h.attach && paid {
                        if players[o].fury <= 0.0 {
                            players[o].gas -= cost;
                        }
                        h.pulling = true;
                        h.start = players[o].pos;
                        events.push(Event::HookBite { pos: h.to });
                    } else {
                        h.back = true;
                        players[o].chain = 0;
                    }
                } else {
                    h.pos += left.normalize() * step;
                }
                return true;
            }
            players[o].lock = players[o].lock.max(2);
            players[o].flying = true;
            let from = players[o].pos;
            let left = h.to - from;
            let step = PULL_SPEED * (1.0 + CHAIN_SPEED * players[o].chain as f32) * DT;
            let done = left.length() <= step;
            let dir = left.normalize_or_zero();
            players[o].pos = if done { h.to } else { from + dir * step };
            if dir != Vec2::ZERO {
                players[o].ang = dir.y.atan2(dir.x);
            }
            players[o].clamp();
            let now = players[o].pos;
            if (now - h.start).length() as u32 % 3 == 0 {
                events.push(Event::Ghost { seat: o, pos: now, ang: players[o].ang, life: 0.25 });
            }
            let t = 1 - o;
            if !h.passed && seg_dist(players[t].pos, from, now) < RADIUS + 10.0 {
                h.passed = true;
                let chain = players[o].chain as i32;
                if players[o].fury > 0.0 || chain > 0 {
                    let dmg = if players[o].fury > 0.0 { ZIP_DMG } else { 0 } + chain * CHAIN_DMG;
                    strike(players, t, Hit::new(power(&players[o], dmg), dir, Element::Blade, true, &[]), events);
                }
                spins.push(o);
            }
            if done {
                players[o].lock = 0;
                players[o].flying = false;
                events.push(Event::Zip { from: h.start, to: now });
            }
            !done
        });
        for (n, p) in players.iter_mut().enumerate() {
            p.hooked = self.hooks.iter().any(|h| h.owner == n && !h.back);
        }
        for o in spins {
            players[o].target = players[o].pos + players[o].facing() * 10.0;
            self.whirl(players, o, true, events);
        }
        self.whirls.retain_mut(|w| {
            w.tick += 1;
            let o = w.owner;
            if players[o].state == crate::core::anim::State::Hit && !w.big {
                return false;
            }
            if !w.big {
                players[o].pos += w.dir * WHIRL_DRIFT / WHIRL_TICKS as f32;
                players[o].clamp();
            }
            let t = 1 - o;
            for (n, at) in WHIRL_HITS.iter().enumerate() {
                if w.tick != *at || w.hit[n] {
                    continue;
                }
                let d = players[t].pos - players[o].pos;
                if d.length() < WHIRL_R + RADIUS {
                    let dmg = power(&players[o], if w.big { WHIRL_DMG + 1 } else { WHIRL_DMG });
                    if strike(players, t, Hit::new(dmg, d.normalize_or(Vec2::X), Element::Blade, w.big, &[]), events) {
                        w.hit[n] = true;
                        if players[o].fury > 0.0 {
                            let max = players[o].max_hp();
                            players[o].hp = (players[o].hp + FURY_HEAL).min(max);
                            events.push(Event::Heal { pos: players[o].pos, amount: FURY_HEAL });
                        }
                    }
                }
                events.push(Event::WhirlPulse { owner: o, big: w.big });
            }
            w.tick < WHIRL_TICKS
        });
    }
}
