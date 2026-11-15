use std::f32::consts::TAU;

use macroquad::{math::Vec2, rand::gen_range};

use crate::core::{
    battle::strike,
    event::{Element, Event},
    inside,
    kit::WIND_CHARGES,
    player::{Player},
    seg_dist, Skill, DT, RADIUS,
};

pub const FORM_RANGE: f32 = 130.0;
pub const FORM_N: usize = 8;
pub const FORM_R: f32 = 36.0;
pub const FORM_APPEAR: f32 = 0.35;
pub const FORM_HOVER: f32 = 0.6;
pub const FORM_GAP: f32 = 0.06;
pub const FORM_STAB: f32 = 0.14;
const FORM_DMG: i32 = 3;

pub const LUNGE_TICKS: u32 = 9;
const LUNGE_DIST: f32 = 52.0;
pub const LUNGE_REACH: f32 = 12.0;
const LUNGE_DMG: i32 = 7;

const WIND_DIST: f32 = 65.0;
const WIND_DMG: i32 = 6;
const WIND_GHOSTS: usize = 4;

pub struct Lunge {
    pub owner: usize,
    pub dir: Vec2,
    tick: u32,
    hit: bool,
}

impl Lunge {
    pub fn ang(&self) -> f32 {
        self.dir.y.atan2(self.dir.x)
    }
}

pub struct Formation {
    pub owner: usize,
    pub center: Vec2,
    pub t: f32,
    pub base: f32,
    hit: [bool; FORM_N],
    done: bool,
}

impl Formation {
    pub fn dir(&self, n: usize) -> Vec2 {
        Vec2::from_angle(self.base + n as f32 * TAU / FORM_N as f32)
    }

    pub fn stab_start(n: usize) -> f32 {
        FORM_HOVER + n as f32 * FORM_GAP
    }

    pub fn stab(&self, n: usize) -> f32 {
        (self.t - Self::stab_start(n)) / FORM_STAB
    }

    pub fn sword_pos(&self, n: usize) -> Vec2 {
        let s = self.stab(n).clamp(0.0, 1.0);
        self.center + self.dir(n) * FORM_R * (1.0 - 2.0 * s)
    }

    fn end() -> f32 {
        Self::stab_start(FORM_N - 1) + FORM_STAB
    }
}

pub struct Cut {
    pub r: f32,
    pub half: f32,
    pub dmg: i32,
}

pub fn cut_shape(combo: u32) -> Cut {
    if combo == 3 {
        Cut { r: 30.0, half: 1.5, dmg: 6 }
    } else {
        Cut { r: 22.0, half: 1.05, dmg: 4 }
    }
}

#[derive(Default)]
pub struct Blades {
    pub lunges: Vec<Lunge>,
    pub forms: Vec<Formation>,
}

impl Blades {
    pub fn cast(&mut self, players: &mut [Player; 2], i: usize, skill: Skill, events: &mut Vec<Event>) {
        let t = 1 - i;
        let f = players[i].facing();
        let pos = players[i].pos;
        match skill {
            Skill::Cut => {
                let combo = players[i].combo;
                let big = combo == 3;
                let c = cut_shape(combo);
                let side = if combo % 2 == 0 { -1.0 } else { 1.0 };
                events.push(Event::Cut { owner: i, ang: players[i].ang, r: c.r, half: c.half, side, big });
                let to = players[t].pos - pos;
                if to.length() < c.r + RADIUS && f.angle_between(to).abs() < c.half {
                    strike(players, t, f, c.dmg, Element::Blade, big, events);
                }
            }
            Skill::Swallow => {
                self.lunges.push(Lunge { owner: i, dir: f, tick: 0, hit: false });
                events.push(Event::Swallow { pos, dir: f });
            }
            Skill::WindStep => {
                let dist = (players[i].target - pos).clamp_length(WIND_DIST * 0.6, WIND_DIST).length();
                players[i].pos = pos + f * dist;
                players[i].clamp();
                let end = players[i].pos;
                let ang = players[i].ang;
                events.push(Event::Ghost { seat: i, pos, ang, life: 0.3 });
                for n in 1..WIND_GHOSTS {
                    let g = pos.lerp(end, n as f32 / WIND_GHOSTS as f32);
                    events.push(Event::Ghost { seat: i, pos: g, ang, life: 0.12 + n as f32 * 0.05 });
                }
                events.push(Event::Wind { from: pos, to: end, dir: f });
                if seg_dist(players[t].pos, pos, end) < RADIUS + 8.0 {
                    strike(players, t, f, WIND_DMG, Element::Blade, false, events);
                }
            }
            Skill::Formation => {
                let center = inside(pos + (players[i].target - pos).clamp_length_max(FORM_RANGE), FORM_R * 0.5);
                self.forms.push(Formation { owner: i, center, t: 0.0, base: gen_range(0.0, TAU), hit: [false; FORM_N], done: false });
                events.push(Event::FormCast { caster: pos, center });
            }
            _ => {}
        }
    }

    pub fn update(&mut self, players: &mut [Player; 2], events: &mut Vec<Event>) {
        self.lunges.retain_mut(|l| {
            l.tick += 1;
            let o = l.owner;
            if players[o].state == crate::core::anim::State::Hit {
                return false;
            }
            players[o].pos += l.dir * LUNGE_DIST / LUNGE_TICKS as f32;
            players[o].clamp();
            if l.tick % 2 == 0 {
                events.push(Event::Ghost { seat: o, pos: players[o].pos, ang: players[o].ang, life: 0.3 });
            }
            let tip = players[o].pos + l.dir * LUNGE_REACH;
            let t = 1 - o;
            if !l.hit && seg_dist(players[t].pos, players[o].pos, tip) < RADIUS + 5.0 && strike(players, t, l.dir, LUNGE_DMG, Element::Blade, true, events) {
                l.hit = true;
                players[o].charges = (players[o].charges + 1).min(WIND_CHARGES);
                events.push(Event::SwallowHit { tip });
            }
            l.tick < LUNGE_TICKS
        });
        self.forms.retain_mut(|fm| {
            fm.t += DT;
            let t = 1 - fm.owner;
            for n in 0..FORM_N {
                if !(0.0..=1.0).contains(&fm.stab(n)) || fm.hit[n] {
                    continue;
                }
                let pos = fm.sword_pos(n);
                if players[t].pos.distance(pos) < RADIUS + 5.0 && strike(players, t, -fm.dir(n), FORM_DMG, Element::Blade, false, events) {
                    fm.hit[n] = true;
                }
            }
            if fm.t >= Formation::end() && !fm.done {
                fm.done = true;
                events.push(Event::FormBurst { center: fm.center, base: fm.base });
            }
            fm.t < Formation::end() + 0.05
        });
    }
}
