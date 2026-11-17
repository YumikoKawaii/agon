use macroquad::math::Vec2;

use crate::core::{
    battle::{strike, Hit},
    effect::{Apply, EffectKind},
    event::{Element, Event},
    magic::{ward_block, Ward},
    player::Player,
    seg_dist, Skill, ARENA, DT, RADIUS,
};

pub const OVER_MAX: u8 = 3;
const OVER_MUL: f32 = 1.4;

pub const SHOT_SPEED: f32 = 300.0;
pub const SHOT_R: f32 = 4.0;
const SHOT_DMG: i32 = 2;
const SHOT_LIFE: f32 = 0.7;

pub const LANCE_RANGE: f32 = 220.0;
pub const LANCE_W: f32 = 4.0;
pub const LANCE_SHOW: f32 = 0.18;
const LANCE_DMG: i32 = 5;
const LANCE_BIG_MUL: f32 = 1.5;

pub const WELL_RANGE: f32 = 140.0;
pub const WELL_R: f32 = 45.0;
pub const WELL_T: f32 = 1.2;
const WELL_PULL: f32 = 60.0;
const WELL_DMG: i32 = 4;
const WELL_FX: &[Apply] = &[Apply(EffectKind::Slow, 18, 30)];

pub const BURST_R: f32 = 50.0;
const BURST_DMG: i32 = 10;

pub const KNUCKLE_R: f32 = 22.0;
pub const KNUCKLE_BIG_R: f32 = 30.0;
const KNUCKLE_HALF: f32 = 1.0;
const KNUCKLE_DMG: i32 = 3;
const KNUCKLE_BIG_DMG: i32 = 5;

pub const FISTS_T: f32 = 4.0;
const FISTS_MUL: f32 = 1.6;
const FISTS_REACH: f32 = 1.25;
pub const WAVE_RANGE: f32 = 60.0;
pub const WAVE_BIG_RANGE: f32 = 100.0;
pub const WAVE_SPEED: f32 = 320.0;
pub const WAVE_R: f32 = 12.0;
const WAVE_DMG: i32 = 3;

pub const DASH_RANGE: f32 = 100.0;
const DASH_HIT: f32 = RADIUS + 6.0;
const DASH_DMG: i32 = 4;
const PRIMED_T: f32 = 1.0;

pub const BEAM_RANGE: f32 = 200.0;
pub const BEAM_W: f32 = 9.0;
pub const BEAM_CHARGE: f32 = 0.8;
pub const BEAM_SHOW: f32 = 0.45;
const BEAM_DMG: i32 = 22;
const BEAM_FX: &[Apply] = &[Apply(EffectKind::Slow, 60, 50)];

const GHOSTS: usize = 6;

pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: usize,
    pub age: f32,
    life: f32,
}

pub struct Lance {
    pub owner: usize,
    pub origin: Vec2,
    pub dir: Vec2,
    pub t: f32,
    pub big: bool,
}

impl Lance {
    pub fn end(&self) -> Vec2 {
        self.origin + self.dir * LANCE_RANGE
    }

    pub fn width(&self) -> f32 {
        if self.big {
            LANCE_W * 2.0
        } else {
            LANCE_W
        }
    }
}

pub struct Well {
    pub owner: usize,
    pub pos: Vec2,
    pub t: f32,
}

pub struct Wave {
    pub owner: usize,
    pub pos: Vec2,
    pub dir: Vec2,
    pub dist: f32,
    pub dmg: i32,
    pub big: bool,
}

impl Wave {
    pub fn range(&self) -> f32 {
        if self.big {
            WAVE_BIG_RANGE
        } else {
            WAVE_RANGE
        }
    }

    pub fn r(&self) -> f32 {
        if self.big {
            WAVE_R * 1.6
        } else {
            WAVE_R
        }
    }
}

pub struct Beam {
    pub owner: usize,
    pub origin: Vec2,
    pub dir: Vec2,
    pub t: f32,
    pub fired: bool,
    pub over: bool,
}

impl Beam {
    pub fn charge(&self) -> f32 {
        (self.t / BEAM_CHARGE).min(1.0)
    }

    pub fn width(&self) -> f32 {
        if self.over {
            BEAM_W * OVER_MUL
        } else {
            BEAM_W
        }
    }

    pub fn end(&self) -> Vec2 {
        self.origin + self.dir * BEAM_RANGE
    }
}

#[derive(Default)]
pub struct Serens {
    pub shots: Vec<Shot>,
    pub lances: Vec<Lance>,
    pub wells: Vec<Well>,
    pub waves: Vec<Wave>,
    pub beams: Vec<Beam>,
}

fn spend(p: &mut Player) -> bool {
    if p.over >= OVER_MAX {
        p.over = 0;
        true
    } else {
        false
    }
}

fn boost(dmg: i32, mul: f32) -> i32 {
    (dmg as f32 * mul).round() as i32
}

fn over_mul(over: bool) -> f32 {
    if over {
        OVER_MUL
    } else {
        1.0
    }
}

impl Serens {
    pub fn reset(&mut self, i: usize) {
        self.shots.retain(|s| s.owner != i);
        self.lances.retain(|l| l.owner != i);
        self.wells.retain(|w| w.owner != i);
        self.waves.retain(|w| w.owner != i);
        self.beams.retain(|b| b.owner != i);
    }

    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let t = 1 - i;
        let f = players[i].facing();
        let pos = players[i].pos;
        let aim = players[i].target;
        let tip = pos + f * 8.0;
        match skill {
            Skill::ManaShot => {
                self.shots.push(Shot { pos: tip, vel: f * SHOT_SPEED, owner: i, age: 0.0, life: SHOT_LIFE });
                events.push(Event::SerenShot { tip, dir: f });
            }
            Skill::AzureLance => {
                let end = tip + f * LANCE_RANGE;
                let big = self.wells.iter().any(|w| w.owner == i && seg_dist(w.pos, tip, end) < WELL_R);
                let lance = Lance { owner: i, origin: tip, dir: f, t: 0.0, big };
                let hit = players[t].hurtbox().is_some_and(|h| seg_dist(players[t].pos, tip, end) < lance.width() + h);
                if hit {
                    let dmg = if big { boost(LANCE_DMG, LANCE_BIG_MUL) } else { LANCE_DMG };
                    strike(players, t, Hit::new(dmg, f, Element::Arcane, big, &[]), events);
                }
                events.push(Event::SerenLance { origin: tip, dir: f, big, hit: if hit { Some(players[t].pos) } else { None } });
                self.lances.push(lance);
            }
            Skill::GravityWell => {
                let at = pos + (aim - pos).clamp_length_max(WELL_RANGE);
                self.wells.retain(|w| w.owner != i);
                self.wells.push(Well { owner: i, pos: at, t: 0.0 });
                events.push(Event::SerenWell { pos: at });
            }
            Skill::Overdrive => {
                let over = spend(&mut players[i]);
                events.push(Event::SerenTransform { pos, over });
                let d = players[t].pos - pos;
                if d.length() < BURST_R + RADIUS {
                    strike(players, t, Hit::new(boost(BURST_DMG, over_mul(over)), d.normalize_or(f), Element::Arcane, true, &[]).knock(), events);
                }
                players[i].armed = false;
            }
            Skill::RuneKnuckle => {
                let big = players[i].combo == 3;
                let lit = players[i].fists > 0.0;
                let (r, dmg) = if big { (KNUCKLE_BIG_R, KNUCKLE_BIG_DMG) } else { (KNUCKLE_R, KNUCKLE_DMG) };
                let (r, dmg) = if lit { (r * FISTS_REACH, boost(dmg, FISTS_MUL)) } else { (r, dmg) };
                events.push(Event::SerenPunch { pos: pos + f * 10.0, ang: players[i].ang, big, lit });
                let rel = players[t].pos - pos;
                if rel.length() < r + RADIUS && f.angle_between(rel).abs() < KNUCKLE_HALF {
                    let hit = Hit::new(dmg, f, Element::Arcane, big, &[]);
                    strike(players, t, if big { hit.knock() } else { hit }, events);
                    if big {
                        events.push(Event::SerenBlast { pos: players[t].pos, big: lit });
                    }
                }
                if lit {
                    self.waves.push(Wave { owner: i, pos: tip, dir: f, dist: 0.0, dmg: if big { WAVE_DMG * 2 } else { WAVE_DMG }, big });
                }
            }
            Skill::OverFists => {
                players[i].fists = FISTS_T;
                events.push(Event::SerenFists { pos });
            }
            Skill::RocketDash => {
                let to = pos + (aim - pos).clamp_length_max(DASH_RANGE);
                players[i].pos = to;
                players[i].clamp();
                let end = players[i].pos;
                let ang = players[i].ang;
                for n in 0..GHOSTS {
                    events.push(Event::Ghost { seat: i, pos: pos.lerp(end, n as f32 / GHOSTS as f32), ang, life: 0.15 + n as f32 * 0.06 });
                }
                events.push(Event::SerenDash { from: pos, to: end });
                if seg_dist(players[t].pos, pos, end) < DASH_HIT {
                    strike(players, t, Hit::new(DASH_DMG, (end - pos).normalize_or(f), Element::Arcane, false, &[]), events);
                }
                players[i].primed = PRIMED_T;
            }
            Skill::Starbreaker => {
                let over = spend(&mut players[i]);
                self.beams.retain(|b| b.owner != i);
                self.beams.push(Beam { owner: i, origin: tip, dir: f, t: 0.0, fired: false, over });
                events.push(Event::SerenCharge { pos: tip });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], wards: &[Ward], events: &mut Vec<Event>) {
        self.shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            s.age += DT;
            if ward_block(wards, players, s.owner, s.pos, SHOT_R, events) {
                return false;
            }
            let t = 1 - s.owner;
            if players[t].hurtbox().is_some_and(|h| players[t].pos.distance(s.pos) < h + SHOT_R) {
                strike(players, t, Hit::new(SHOT_DMG, s.vel.normalize_or_zero(), Element::Arcane, false, &[]), events);
                return false;
            }
            s.life > 0.0 && s.pos.cmpge(Vec2::ZERO).all() && s.pos.cmple(ARENA).all()
        });
        self.lances.retain_mut(|l| {
            l.t += DT;
            l.t < LANCE_SHOW
        });
        self.wells.retain_mut(|w| {
            w.t += DT;
            let t = 1 - w.owner;
            let d = w.pos - players[t].pos;
            if players[t].hurtbox().is_some() && d.length() < WELL_R + RADIUS && d.length() > 2.0 {
                players[t].pos += d.normalize() * WELL_PULL * DT;
                players[t].clamp();
            }
            if w.t < WELL_T {
                return true;
            }
            events.push(Event::SerenPop { pos: w.pos });
            if players[t].pos.distance(w.pos) < WELL_R + RADIUS {
                strike(players, t, Hit::new(WELL_DMG, (players[t].pos - w.pos).normalize_or(Vec2::X), Element::Arcane, true, WELL_FX), events);
            }
            false
        });
        self.waves.retain_mut(|w| {
            let step = WAVE_SPEED * DT;
            w.pos += w.dir * step;
            w.dist += step;
            let t = 1 - w.owner;
            if players[t].hurtbox().is_some_and(|h| players[t].pos.distance(w.pos) < h + w.r()) {
                strike(players, t, Hit::new(w.dmg, w.dir, Element::Arcane, false, &[]), events);
                events.push(Event::SerenBlast { pos: w.pos, big: w.big });
                return false;
            }
            w.dist < w.range()
        });
        self.beams.retain_mut(|b| {
            b.t += DT;
            let i = b.owner;
            if !b.fired && b.t >= BEAM_CHARGE {
                b.fired = true;
                events.push(Event::SerenBeam { origin: b.origin, dir: b.dir, over: b.over });
                let t = 1 - i;
                if seg_dist(players[t].pos, b.origin, b.end()) < b.width() + RADIUS {
                    strike(players, t, Hit::new(boost(BEAM_DMG, over_mul(b.over)), b.dir, Element::Arcane, true, BEAM_FX).knock(), events);
                }
                players[i].armed = true;
            }
            b.t < BEAM_CHARGE + BEAM_SHOW
        });
    }
}
