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

pub const ORB_SPEED: f32 = 230.0;
pub const ORB_R: f32 = 4.0;
const ORB_DMG: i32 = 2;
const ORB_LIFE: f32 = 0.85;

pub const BLIND_SPEED: f32 = 200.0;
pub const BLIND_R: f32 = 5.0;
const BLIND_LIFE: f32 = 0.9;
pub const BLIND_BLAST: f32 = 30.0;
const BLIND_DMG: i32 = 5;
const BLIND_FX: &[Apply] = &[Apply(EffectKind::Stun, 40, 0)];

pub const SHINE_RANGE: f32 = 150.0;
pub const SHINE_R: f32 = 34.0;
pub const SHINE_T: f32 = 0.5;
const SHINE_DMG: i32 = 7;

pub const CHARGE_RANGE: f32 = 100.0;
const CHARGE_HIT: f32 = RADIUS + 8.0;
const CHARGE_DMG: i32 = 5;
const CHARGE_FX: &[Apply] = &[Apply(EffectKind::Slow, 120, 30)];

pub const CLAW_R: f32 = 22.0;
const CLAW_HALF: f32 = 1.1;
const CLAW_DMG: i32 = 2;

pub const WISP_SPEED: f32 = 220.0;
pub const WISP_R: f32 = 4.0;
pub const WISP_RANGE: f32 = 176.0;
const WISP_DMG: i32 = 2;
const WISP_LIFE: f32 = 0.8;
const WISP_SPREAD: f32 = 0.22;
const WISP_SLOW: Apply = Apply(EffectKind::Slow, 60, 40);

pub const SWIPE_R: f32 = 26.0;
const SWIPE_HALF: f32 = 1.0;
const SWIPE_DMG: i32 = 2;
const SWIPE_HITS: [u32; 3] = [0, 8, 16];
const SWIPE_STEP: f32 = 4.0;


const GHOSTS: usize = 6;

pub struct Orb {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: usize,
    pub age: f32,
    pub kind: OrbKind,
    volley: u32,
    life: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrbKind {
    Spirit,
    Blind,
    Wisp,
}

impl OrbKind {
    pub fn r(self) -> f32 {
        match self {
            OrbKind::Spirit => ORB_R,
            OrbKind::Blind => BLIND_R,
            OrbKind::Wisp => WISP_R,
        }
    }
}

pub struct Swipe {
    pub owner: usize,
    pub tick: u32,
    hit: [bool; 3],
}

pub struct Shine {
    pub owner: usize,
    pub pos: Vec2,
    pub t: f32,
}

impl Shine {
    pub fn k(&self) -> f32 {
        (self.t / SHINE_T).min(1.0)
    }
}

#[derive(Default)]
pub struct Foxes {
    pub orbs: Vec<Orb>,
    pub shines: Vec<Shine>,
    pub swipes: Vec<Swipe>,
    volleys: u32,
    tally: Vec<(u32, u8)>,
}

fn dash(players: &mut [Player; 2], i: usize, to: Vec2, events: &mut Vec<Event>) -> (Vec2, Vec2) {
    let from = players[i].pos;
    players[i].pos = to;
    players[i].clamp();
    let end = players[i].pos;
    let ang = players[i].ang;
    for n in 0..GHOSTS {
        let g = from.lerp(end, n as f32 / GHOSTS as f32);
        events.push(Event::Ghost { seat: i, pos: g, ang, life: 0.15 + n as f32 * 0.06 });
    }
    events.push(Event::FoxDash { from, to: end });
    (from, end)
}

fn cone(players: &mut [Player; 2], i: usize, r: f32, half: f32, hit: Hit, events: &mut Vec<Event>) {
    let t = 1 - i;
    let rel = players[t].pos - players[i].pos;
    if rel.length() < r + RADIUS && players[i].facing().angle_between(rel).abs() < half {
        strike(players, t, hit, events);
    }
}

impl Foxes {
    pub fn reset(&mut self, i: usize) {
        self.orbs.retain(|o| o.owner != i);
        self.shines.retain(|s| s.owner != i);
        self.swipes.retain(|s| s.owner != i);
    }

    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let t = 1 - i;
        let f = players[i].facing();
        let pos = players[i].pos;
        let aim = players[i].target;
        match skill {
            Skill::FoxOrb | Skill::BlindLight => {
                let (kind, speed, life) = if skill == Skill::BlindLight { (OrbKind::Blind, BLIND_SPEED, BLIND_LIFE) } else { (OrbKind::Spirit, ORB_SPEED, ORB_LIFE) };
                let tip = pos + f * 8.0;
                self.orbs.push(Orb { pos: tip, vel: f * speed, owner: i, age: 0.0, kind, volley: 0, life });
                events.push(Event::FoxCast { tip, dir: f });
            }
            Skill::ShiningLight => {
                let at = inside(pos + (aim - pos).clamp_length_max(SHINE_RANGE), SHINE_R * 0.5);
                self.shines.push(Shine { owner: i, pos: at, t: 0.0 });
            }
            Skill::Unpredictable | Skill::Vanish => {
                events.push(Event::FoxPoof { pos });
                let (from, end) = dash(players, i, pos + (aim - pos).clamp_length_max(CHARGE_RANGE), events);
                if seg_dist(players[t].pos, from, end) < CHARGE_HIT {
                    let dir = (end - from).normalize_or(f);
                    strike(players, t, Hit::new(CHARGE_DMG, dir, Element::Spirit, true, CHARGE_FX), events);
                }
                players[i].armed = skill == Skill::Vanish;
                events.push(Event::FoxPoof { pos: players[i].pos });
            }
            Skill::FoxClaw => {
                events.push(Event::FoxSwipe { pos, ang: players[i].ang, big: false });
                cone(players, i, CLAW_R, CLAW_HALF, Hit::new(CLAW_DMG, f, Element::Spirit, false, &[]), events);
            }
            Skill::FoxWisp => {
                self.volleys += 1;
                let volley = self.volleys;
                self.tally.push((volley, 0));
                let tip = pos + f * 8.0;
                for off in [-WISP_SPREAD, 0.0, WISP_SPREAD] {
                    let d = Vec2::from_angle(players[i].ang + off);
                    self.orbs.push(Orb { pos: tip, vel: d * WISP_SPEED, owner: i, age: 0.0, kind: OrbKind::Wisp, volley, life: WISP_LIFE });
                }
                events.push(Event::FoxCast { tip, dir: f });
            }
            Skill::FoxSwipe => {
                self.swipes.retain(|s| s.owner != i);
                self.swipes.push(Swipe { owner: i, tick: 0, hit: [false; 3] });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], wards: &[Ward], events: &mut Vec<Event>) {
        let tally = &mut self.tally;
        self.orbs.retain_mut(|o| {
            o.pos += o.vel * DT;
            o.life -= DT;
            o.age += DT;
            let r = o.kind.r();
            if ward_block(wards, players, o.owner, o.pos, r, events) {
                return false;
            }
            let t = 1 - o.owner;
            let hit = players[t].hurtbox().is_some_and(|h| players[t].pos.distance(o.pos) < h + r);
            let alive = o.life > 0.0 && o.pos.cmpge(Vec2::ZERO).all() && o.pos.cmple(ARENA).all();
            if o.kind == OrbKind::Blind && (hit || !alive) {
                events.push(Event::FoxBlast { pos: o.pos });
                let d = players[t].pos - o.pos;
                if d.length() < BLIND_BLAST + RADIUS {
                    strike(players, t, Hit::new(BLIND_DMG, d.normalize_or(Vec2::X), Element::Spirit, true, BLIND_FX), events);
                }
                return false;
            }
            if hit {
                let dmg = if o.kind == OrbKind::Wisp { WISP_DMG } else { ORB_DMG };
                if strike(players, t, Hit::new(dmg, o.vel.normalize_or_zero(), Element::Spirit, false, &[]), events) && o.kind == OrbKind::Wisp {
                    if let Some(v) = tally.iter_mut().find(|v| v.0 == o.volley) {
                        v.1 += 1;
                        if v.1 == 3 {
                            players[t].effects.add(WISP_SLOW);
                            events.push(Event::Status { pos: players[t].pos, kind: EffectKind::Slow });
                            events.push(Event::FoxBlast { pos: players[t].pos });
                        }
                    }
                }
            }
            !hit && alive
        });
        let live = &self.orbs;
        self.tally.retain(|v| live.iter().any(|o| o.volley == v.0));
        self.swipes.retain_mut(|s| {
            let i = s.owner;
            for (n, at) in SWIPE_HITS.iter().enumerate() {
                if s.tick == *at && !s.hit[n] {
                    s.hit[n] = true;
                    let f = players[i].facing();
                    players[i].pos += f * SWIPE_STEP;
                    players[i].clamp();
                    events.push(Event::FoxSwipe { pos: players[i].pos, ang: players[i].ang + if n % 2 == 0 { 0.35 } else { -0.35 }, big: n == 2 });
                    cone(players, i, SWIPE_R, SWIPE_HALF, Hit::new(SWIPE_DMG, f, Element::Spirit, n == 2, &[]), events);
                }
            }
            s.tick += 1;
            s.tick <= SWIPE_HITS[2]
        });
        self.shines.retain_mut(|s| {
            s.t += DT;
            if s.t < SHINE_T {
                return true;
            }
            events.push(Event::FoxStrike { pos: s.pos });
            let t = 1 - s.owner;
            let d = players[t].pos - s.pos;
            if d.length() < SHINE_R + RADIUS {
                strike(players, t, Hit::new(SHINE_DMG, d.normalize_or(Vec2::X), Element::Spirit, true, &[]), events);
            }
            false
        });
    }
}
