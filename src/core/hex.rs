use macroquad::math::Vec2;

use crate::core::{
    battle::strike,
    event::{Element, Event},
    inside,
    player::Player,
    seg_dist, Skill, ARENA, DT, RADIUS,
};

pub const BOLT_SPEED: f32 = 190.0;
pub const BOLT_R: f32 = 4.5;
const BOLT_DMG: i32 = 12;
const BOLT_LIFE: f32 = 1.2;
const BOLT_TIP: f32 = 8.0;

pub const POTION_RANGE: f32 = 150.0;
pub const POTION_T: f32 = 0.5;
pub const POTION_R: f32 = 22.0;
const POTION_DMG: i32 = 22;

pub const BROOM_DIST: f32 = 80.0;
const BROOM_DMG: i32 = 15;
const BROOM_GHOSTS: usize = 4;

pub const CAULDRON_RANGE: f32 = 140.0;
pub const CAULDRON_R: f32 = 45.0;
pub const CAULDRON_T: f32 = 0.8;
const CAULDRON_DMG: i32 = 40;

pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: usize,
    life: f32,
}

pub struct Potion {
    pub from: Vec2,
    pub to: Vec2,
    pub t: f32,
    pub owner: usize,
}

impl Potion {
    pub fn k(&self) -> f32 {
        (self.t / POTION_T).min(1.0)
    }
}

pub struct Cauldron {
    pub pos: Vec2,
    pub t: f32,
    pub owner: usize,
}

impl Cauldron {
    pub fn charge(&self) -> f32 {
        (self.t / CAULDRON_T).min(1.0)
    }
}

fn target(p: &Player, range: f32, margin: f32) -> Vec2 {
    inside(p.pos + (p.target - p.pos).clamp_length_max(range), margin)
}

#[derive(Default)]
pub struct Hexes {
    pub bolts: Vec<Bolt>,
    pub potions: Vec<Potion>,
    pub cauldrons: Vec<Cauldron>,
}

impl Hexes {
    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let p = &players[i];
        let f = p.facing();
        let pos = p.pos;
        match skill {
            Skill::Hex => {
                let tip = pos + f * BOLT_TIP;
                self.bolts.push(Bolt { pos: tip, vel: f * BOLT_SPEED, owner: i, life: BOLT_LIFE });
                events.push(Event::HexCast { tip });
            }
            Skill::Potion => {
                let to = target(p, POTION_RANGE, POTION_R * 0.5);
                self.potions.push(Potion { from: pos, to, t: 0.0, owner: i });
            }
            Skill::Broom => {
                let dist = (p.target - pos).clamp_length(BROOM_DIST * 0.6, BROOM_DIST).length();
                let ang = p.ang;
                players[i].pos = pos + f * dist;
                players[i].clamp();
                let end = players[i].pos;
                for n in 0..BROOM_GHOSTS {
                    let g = pos.lerp(end, n as f32 / BROOM_GHOSTS as f32);
                    events.push(Event::Ghost { seat: i, pos: g, ang, life: 0.12 + n as f32 * 0.05 });
                }
                events.push(Event::Broom { from: pos, to: end, dir: f });
                let t = 1 - i;
                if seg_dist(players[t].pos, pos, end) < RADIUS + 8.0 {
                    strike(players, t, f, BROOM_DMG, Element::Hex, false, events);
                }
            }
            Skill::Cauldron => {
                let at = target(p, CAULDRON_RANGE, CAULDRON_R * 0.6);
                self.cauldrons.push(Cauldron { pos: at, t: 0.0, owner: i });
                events.push(Event::CauldronCast { caster: pos, at });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        self.bolts.retain_mut(|b| {
            b.pos += b.vel * DT;
            b.life -= DT;
            let t = 1 - b.owner;
            let hit = players[t].hurtbox().is_some_and(|r| players[t].pos.distance(b.pos) < r + BOLT_R);
            if hit {
                strike(players, t, b.vel.normalize_or_zero(), BOLT_DMG, Element::Hex, false, events);
            }
            let alive = b.pos.cmpge(Vec2::ZERO).all() && b.pos.cmple(ARENA).all();
            !hit && b.life > 0.0 && alive
        });
        self.potions.retain_mut(|p| {
            p.t += DT;
            if p.t < POTION_T {
                return true;
            }
            let t = 1 - p.owner;
            let d = players[t].pos - p.to;
            if d.length() < POTION_R + RADIUS {
                strike(players, t, d.normalize_or(Vec2::X), POTION_DMG, Element::Hex, true, events);
            }
            events.push(Event::PotionBurst { pos: p.to });
            false
        });
        self.cauldrons.retain_mut(|c| {
            c.t += DT;
            if c.t < CAULDRON_T {
                return true;
            }
            let t = 1 - c.owner;
            let d = players[t].pos - c.pos;
            if d.length() < CAULDRON_R + RADIUS {
                strike(players, t, d.normalize_or(Vec2::X) * 4.0 / 3.0, CAULDRON_DMG, Element::Hex, true, events);
            }
            events.push(Event::CauldronBurst { pos: c.pos });
            false
        });
    }
}
