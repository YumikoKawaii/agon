use std::collections::VecDeque;

use macroquad::{
    math::{vec2, Vec2},
    rand::gen_range,
};

use crate::core::{
    magic::{JUDGE_R, JUDGE_RANGE, RAY_LEN, RAY_W},
    seg_dist,
    sword::{FORM_R, FORM_RANGE},
    Battle, Input, Skill, DT, RADIUS,
};

#[derive(Clone, Copy)]
pub struct Level {
    pub reaction_ticks: usize,
    pub aim_error: f32,
    pub lead: f32,
    pub dodge_read: f32,
    pub hesitate: f32,
    pub aggression: f32,
}

pub const EASY: Level = Level { reaction_ticks: 24, aim_error: 26.0, lead: 0.0, dodge_read: 0.15, hesitate: 0.08, aggression: 0.35 };
pub const NORMAL: Level = Level { reaction_ticks: 15, aim_error: 14.0, lead: 0.5, dodge_read: 0.45, hesitate: 0.03, aggression: 0.6 };
pub const HARD: Level = Level { reaction_ticks: 9, aim_error: 6.0, lead: 1.0, dodge_read: 0.8, hesitate: 0.01, aggression: 0.85 };
pub const LEVELS: [(&str, Level); 3] = [("easy", EASY), ("normal", NORMAL), ("hard", HARD)];

#[derive(Clone, Copy)]
struct Seen {
    pos: Vec2,
    vel: Vec2,
}

pub struct Bot {
    pub level: Level,
    seat: usize,
    memory: VecDeque<Seen>,
    last: Option<Vec2>,
    strafe: f32,
    strafe_ticks: u32,
    idle_ticks: u32,
    dodge_cd: u32,
}

impl Bot {
    pub fn new(level: Level, seat: usize) -> Self {
        Bot { level, seat, memory: VecDeque::new(), last: None, strafe: 1.0, strafe_ticks: 0, idle_ticks: 0, dodge_cd: 0 }
    }

    pub fn think(&mut self, b: &Battle) -> Input {
        let me = &b.players[self.seat];
        let foe = &b.players[1 - self.seat];
        let vel = self.last.map_or(Vec2::ZERO, |p| (foe.pos - p) / DT);
        self.last = Some(foe.pos);
        self.memory.push_back(Seen { pos: foe.pos, vel });
        while self.memory.len() > self.level.reaction_ticks + 1 {
            self.memory.pop_front();
        }
        let seen = self.memory[0];
        self.dodge_cd = self.dodge_cd.saturating_sub(1);

        if self.idle_ticks > 0 {
            self.idle_ticks -= 1;
            return Input { aim: seen.pos, ..Input::default() };
        }
        if gen_range(0.0, 1.0) < self.level.hesitate {
            self.idle_ticks = gen_range(6, 20);
        }

        let to = seen.pos - me.pos;
        let dist = to.length();
        let away = -to.normalize_or_zero();

        if let Some(threat) = self.danger(b) {
            if self.dodge_cd == 0 && gen_range(0.0, 1.0) < self.level.dodge_read {
                self.dodge_cd = 40;
                let side = threat.perp() * self.strafe;
                if let Some(slot) = self.slot_of(b, Skill::Ward) {
                    return Input { aim: me.pos - threat * 20.0, slot: Some(slot), ..Input::default() };
                }
                if let Some(slot) = self.slot_of(b, Skill::WindStep) {
                    return Input { aim: me.pos + side * 60.0, slot: Some(slot), ..Input::default() };
                }
                return Input { dir: side, aim: me.pos + side * 40.0, dodge: me.can_dash(), ..Input::default() };
            }
        }

        if let Some((slot, aim)) = self.pick_skill(b, seen, dist) {
            return Input { dir: Vec2::ZERO, aim, slot: Some(slot), dodge: false };
        }

        if self.strafe_ticks == 0 {
            self.strafe = if gen_range(0, 2) == 0 { 1.0 } else { -1.0 };
            self.strafe_ticks = gen_range(30, 90);
        }
        self.strafe_ticks -= 1;
        let range = self.range(b);
        let side = to.normalize_or_zero().perp() * self.strafe * 0.6;
        let dir = if dist > range + 15.0 {
            to.normalize_or_zero() + side * 0.5
        } else if dist < range - 15.0 {
            away + side
        } else {
            side
        };
        Input { dir, aim: seen.pos, ..Input::default() }
    }

    fn range(&self, b: &Battle) -> f32 {
        let me = &b.players[self.seat];
        if me.kit().skills[0] == Skill::Cut {
            14.0 + (1.0 - self.level.aggression) * 30.0
        } else {
            150.0 - self.level.aggression * 50.0
        }
    }

    fn slot_of(&self, b: &Battle, skill: Skill) -> Option<usize> {
        let me = &b.players[self.seat];
        me.kit().skills.iter().position(|s| *s == skill).filter(|i| me.ready(*i))
    }

    fn aim_at(&self, b: &Battle, seen: Seen, speed: Option<f32>, delay: f32) -> Vec2 {
        let me = &b.players[self.seat];
        let t = speed.map_or(delay, |s| (seen.pos - me.pos).length() / s + delay);
        let jitter = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)) * self.level.aim_error;
        seen.pos + seen.vel * t * self.level.lead + jitter
    }

    fn pick_skill(&self, b: &Battle, seen: Seen, dist: f32) -> Option<(usize, Vec2)> {
        let me = &b.players[self.seat];
        let cast_t = |s: Skill| s.cast_ticks() as f32 * DT * 0.5;
        let wants = |s: Skill| -> bool {
            match s {
                Skill::Needle => dist < 220.0,
                Skill::Ray => dist < RAY_LEN - 30.0 && gen_range(0.0, 1.0) < self.level.aggression * 0.05,
                Skill::Judgement => dist < JUDGE_RANGE && gen_range(0.0, 1.0) < self.level.aggression * 0.04,
                Skill::Cut => dist < 24.0,
                Skill::Swallow => dist < 45.0 && gen_range(0.0, 1.0) < self.level.aggression * 0.08,
                Skill::WindStep => dist > 80.0 && me.charges > 1 && gen_range(0.0, 1.0) < self.level.aggression * 0.02,
                Skill::Formation => dist < FORM_RANGE && gen_range(0.0, 1.0) < self.level.aggression * 0.03,
                Skill::Ward => false,
            }
        };
        let order = [3, 1, 2, 0];
        order.into_iter().filter(|i| me.ready(*i)).map(|i| (i, me.kit().skills[i])).find(|(_, s)| wants(*s)).map(|(i, s)| {
            let speed = match s {
                Skill::Needle => Some(crate::core::magic::NEEDLE_SPEED),
                _ => None,
            };
            (i, self.aim_at(b, seen, speed, cast_t(s)))
        })
    }

    fn danger(&self, b: &Battle) -> Option<Vec2> {
        let me = &b.players[self.seat];
        let foe = 1 - self.seat;
        let pad = RADIUS + 6.0;
        for s in b.spells.shots.iter().filter(|s| s.owner == foe) {
            let ahead = s.pos + s.vel * 0.35;
            if seg_dist(me.pos, s.pos, ahead) < pad {
                return Some(s.vel.normalize_or_zero());
            }
        }
        for r in b.spells.rays.iter().filter(|r| r.owner == foe && !r.fired) {
            if seg_dist(me.pos, r.origin, r.end()) < RAY_W + pad {
                return Some(r.dir);
            }
        }
        for j in b.spells.judges.iter().filter(|j| j.owner == foe) {
            if me.pos.distance(j.pos) < JUDGE_R + RADIUS {
                return Some((me.pos - j.pos).normalize_or_zero().perp());
            }
        }
        for f in b.blades.forms.iter().filter(|f| f.owner == foe) {
            if me.pos.distance(f.center) < FORM_R + RADIUS {
                return Some((me.pos - f.center).normalize_or_zero().perp());
            }
        }
        None
    }
}
