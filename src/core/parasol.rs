use macroquad::math::Vec2;

use crate::core::{
    battle::{strike, Hit},
    effect::{Apply, EffectKind},
    event::{Element, Event},
    inside,
    magic::{ward_block, Ward},
    player::Player,
    seg_dist, Skill, ARENA, DT, RADIUS,
};

pub const TALISMAN_SPEED: f32 = 220.0;
pub const TALISMAN_R: f32 = 4.0;
const TALISMAN_DMG: i32 = 2;
const TALISMAN_LIFE: f32 = 1.0;
const TALISMAN_TIP: f32 = 8.0;

pub const FLIGHT_RANGE: f32 = 140.0;
pub const FLIGHT_SPEED: f32 = 300.0;
const FLIGHT_DMG: i32 = 10;
const FLIGHT_HIT: f32 = RADIUS + 7.0;
const PICKUP_R: f32 = RADIUS + 6.0;
const PICKUP_DELAY: f32 = 0.4;
pub const LEASH: f32 = 200.0;
const RETURN_SPEED: f32 = 380.0;

const STEP_BLINK: f32 = 60.0;
pub const WAVE_R: f32 = 52.0;
pub const WAVE_HALF: f32 = 0.6;
const WAVE_DMG: i32 = 16;
const WAVE_FX: &[Apply] = &[Apply(EffectKind::Slow, 60, 40)];
pub const BOND_R: f32 = 70.0;
const BOND_EVERY: f32 = 1.0;
const BOND_SHIELD: Apply = Apply(EffectKind::Shield, 90, 2);
const BOND_CAP: i32 = 10;
const STEP_GHOSTS: usize = 6;

pub const STORM_RANGE: f32 = 150.0;
pub const STORM_R: f32 = 50.0;
pub const STORM_T: f32 = 1.6;
const STORM_TICK: f32 = 0.3;
const STORM_TICK_DMG: i32 = 3;
const STORM_PULL: f32 = 34.0;
const STORM_DMG: i32 = 22;
const STORM_FX: &[Apply] = &[Apply(EffectKind::Stun, 45, 0)];

pub struct Talisman {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: usize,
    pub age: f32,
    life: f32,
}

pub struct Flight {
    pub owner: usize,
    pub pos: Vec2,
    pub to: Vec2,
    pub spin: f32,
    pub back: bool,
    hit: bool,
}

pub struct Storm {
    pub owner: usize,
    pub pos: Vec2,
    pub t: f32,
    tick: f32,
}

impl Storm {
    pub fn k(&self) -> f32 {
        (self.t / STORM_T).min(1.0)
    }
}

#[derive(Default)]
pub struct Parasols {
    pub talismans: Vec<Talisman>,
    pub flights: Vec<Flight>,
    pub storms: Vec<Storm>,
    pub planted: [Option<Vec2>; 2],
    planted_t: [f32; 2],
    bond_t: [f32; 2],
}

impl Parasols {
    pub fn out(&self, i: usize) -> bool {
        self.planted[i].is_some() || self.flights.iter().any(|f| f.owner == i)
    }

    pub fn anchor(&self, i: usize) -> Option<Vec2> {
        self.planted[i].or_else(|| self.flights.iter().find(|f| f.owner == i).map(|f| f.pos))
    }

    pub fn reset(&mut self, i: usize) {
        self.take(i);
        self.talismans.retain(|t| t.owner != i);
        self.storms.retain(|s| s.owner != i);
    }

    fn take(&mut self, i: usize) {
        self.planted[i] = None;
        self.flights.retain(|f| f.owner != i);
    }

    fn plant(&mut self, i: usize, at: Vec2) {
        self.planted[i] = Some(at);
        self.planted_t[i] = 0.0;
    }

    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let t = 1 - i;
        let f = players[i].facing();
        let pos = players[i].pos;
        let aim = players[i].target;
        match skill {
            Skill::Talisman => {
                let tip = pos + f * TALISMAN_TIP;
                self.talismans.push(Talisman { pos: tip, vel: f * TALISMAN_SPEED, owner: i, age: 0.0, life: TALISMAN_LIFE });
                events.push(Event::TalismanCast { tip, dir: f });
            }
            Skill::Parasol => {
                let from = self.anchor(i).unwrap_or(pos);
                self.take(i);
                let to = inside(from + (aim - from).clamp_length_max(FLIGHT_RANGE), 6.0);
                self.flights.push(Flight { owner: i, pos: from, to, spin: 0.0, back: false, hit: false });
                events.push(Event::ParasolThrow { from, to });
            }
            Skill::SpiritStep => {
                let end = match self.anchor(i) {
                    Some(p) => {
                        self.take(i);
                        p
                    }
                    None => pos + (aim - pos).clamp_length_max(STEP_BLINK),
                };
                players[i].pos = end;
                players[i].clamp();
                let end = players[i].pos;
                let to = aim - end;
                if to.length() > 0.5 {
                    players[i].ang = to.y.atan2(to.x);
                }
                let ang = players[i].ang;
                let dir = players[i].facing();
                for n in 0..STEP_GHOSTS {
                    let g = pos.lerp(end, n as f32 / STEP_GHOSTS as f32);
                    events.push(Event::Ghost { seat: i, pos: g, ang, life: 0.15 + n as f32 * 0.06 });
                }
                events.push(Event::SpiritStep { from: pos, to: end, ang });
                let rel = players[t].pos - end;
                if rel.length() < WAVE_R + RADIUS && dir.angle_between(rel).abs() < WAVE_HALF {
                    strike(players, t, Hit::new(WAVE_DMG, dir, Element::Spirit, true, WAVE_FX), events);
                }
            }
            Skill::Storm => {
                let at = match self.anchor(i) {
                    Some(p) => p,
                    None => {
                        let at = inside(pos + (aim - pos).clamp_length_max(STORM_RANGE), STORM_R * 0.5);
                        events.push(Event::ParasolThrow { from: pos, to: at });
                        at
                    }
                };
                self.take(i);
                self.plant(i, at);
                self.storms.push(Storm { owner: i, pos: at, t: 0.0, tick: 0.0 });
                events.push(Event::StormCast { caster: pos, at });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], wards: &[Ward], events: &mut Vec<Event>) {
        self.talismans.retain_mut(|b| {
            b.pos += b.vel * DT;
            b.life -= DT;
            b.age += DT;
            if ward_block(wards, players, b.owner, b.pos, TALISMAN_R, events) {
                return false;
            }
            let t = 1 - b.owner;
            let hit = players[t].hurtbox().is_some_and(|r| players[t].pos.distance(b.pos) < r + TALISMAN_R);
            if hit {
                strike(players, t, Hit::new(TALISMAN_DMG, b.vel.normalize_or_zero(), Element::Spirit, false, &[]), events);
            }
            let alive = b.pos.cmpge(Vec2::ZERO).all() && b.pos.cmple(ARENA).all();
            !hit && b.life > 0.0 && alive
        });
        let mut landed = Vec::new();
        self.flights.retain_mut(|fl| {
            if fl.back {
                fl.to = players[fl.owner].pos;
            }
            let step = if fl.back { RETURN_SPEED } else { FLIGHT_SPEED } * DT;
            let to = fl.to - fl.pos;
            let prev = fl.pos;
            fl.pos = if to.length() <= step { fl.to } else { fl.pos + to.normalize() * step };
            fl.spin += 18.0 * DT;
            let t = 1 - fl.owner;
            if !fl.back && !fl.hit && seg_dist(players[t].pos, prev, fl.pos) < FLIGHT_HIT {
                let dir = (fl.to - prev).normalize_or(Vec2::X);
                if strike(players, t, Hit::new(FLIGHT_DMG, dir, Element::Spirit, false, &[]), events) {
                    fl.hit = true;
                }
            }
            if fl.back && fl.pos == fl.to {
                refresh_step(&mut players[fl.owner]);
                events.push(Event::ParasolPickup { pos: fl.to });
                return false;
            }
            if fl.pos == fl.to {
                landed.push((fl.owner, fl.to));
                events.push(Event::ParasolPlant { pos: fl.to });
                return false;
            }
            true
        });
        for (i, at) in landed {
            self.plant(i, at);
        }
        for i in 0..2 {
            self.planted_t[i] += DT;
            let near = self.anchor(i).is_some_and(|a| players[i].pos.distance(a) < BOND_R);
            players[i].bond = near;
            self.bond_t[i] = if near { self.bond_t[i] + DT } else { 0.0 };
            if self.bond_t[i] >= BOND_EVERY {
                self.bond_t[i] -= BOND_EVERY;
                if players[i].effects.power[EffectKind::Shield.i()] < BOND_CAP {
                    players[i].effects.add(BOND_SHIELD);
                }
            }
            let storming = self.storms.iter().any(|s| s.owner == i);
            if let Some(at) = self.planted[i] {
                let dist = players[i].pos.distance(at);
                if !storming && self.planted_t[i] > PICKUP_DELAY && dist < PICKUP_R {
                    self.planted[i] = None;
                    events.push(Event::ParasolPickup { pos: at });
                    refresh_step(&mut players[i]);
                } else if !storming && dist > LEASH {
                    self.planted[i] = None;
                    self.flights.push(Flight { owner: i, pos: at, to: players[i].pos, spin: 0.0, back: true, hit: false });
                    events.push(Event::ParasolThrow { from: at, to: players[i].pos });
                }
            }
        }
        self.storms.retain_mut(|s| {
            s.t += DT;
            s.tick += DT;
            let t = 1 - s.owner;
            let d = s.pos - players[t].pos;
            let inside_storm = d.length() < STORM_R + RADIUS;
            if inside_storm && players[t].hurtbox().is_some() && d.length() > 2.0 {
                players[t].pos += d.normalize() * STORM_PULL * DT;
                players[t].clamp();
            }
            if s.tick >= STORM_TICK {
                s.tick -= STORM_TICK;
                events.push(Event::StormTick);
                if inside_storm {
                    strike(players, t, Hit::new(STORM_TICK_DMG, Vec2::ZERO, Element::Spirit, false, &[]), events);
                }
            }
            if s.t < STORM_T {
                return true;
            }
            let d = players[t].pos - s.pos;
            if d.length() < STORM_R + RADIUS {
                strike(players, t, Hit::new(STORM_DMG, d.normalize_or(Vec2::X) * 1.5, Element::Spirit, true, STORM_FX), events);
            }
            events.push(Event::StormBurst { pos: s.pos });
            false
        });
    }
}

fn refresh_step(p: &mut Player) {
    if let Some(slot) = p.kit().skills.iter().position(|k| *k == Skill::SpiritStep) {
        p.cd[slot] = 0.0;
    }
}
