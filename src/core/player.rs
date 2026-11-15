use std::f32::consts::PI;

use macroquad::math::{vec2, Vec2};

use crate::core::{
    anim::{Frame, State},
    kit::{kit, Kit, Skill, WIND_CHARGES},
    ARENA, DT, RADIUS,
};

const SPEED: f32 = 72.0;
const ROLL_SPEED: f32 = 160.0;
pub const DASH_CHARGES: u32 = 2;
pub const DASH_GAP: f32 = 2.0;
pub const DASH_RECHARGE: f32 = 10.0;
const KNOCK_DECAY: f32 = 0.85;
const QUICK_CAST_AGE: u32 = 6;
const COMBO_WINDOW: f32 = 1.0;
const COMBO_FINISHER_CD: f32 = 1.0;

#[derive(Default, Clone, Copy, Debug)]
pub struct Input {
    pub dir: Vec2,
    pub aim: Vec2,
    pub slot: Option<usize>,
    pub dodge: bool,
}

pub struct Player {
    pub pos: Vec2,
    pub ang: f32,
    pub hp: i32,
    pub kit: usize,
    pub state: State,
    pub frame: usize,
    pub age: u32,
    pub cd: [f32; 4],
    pub cast_ticks: u32,
    pub lock: u32,
    pub combo: u32,
    pub charges: u32,
    pub dashes: u32,
    pub dash_gap: f32,
    pub dash_cd: f32,
    pub untargetable: bool,
    pub target: Vec2,
    pub(crate) cast: Option<Skill>,
    knock: Vec2,
    timer: u32,
    finished: bool,
    combo_timer: f32,
    spawn: (Vec2, f32),
}

impl Player {
    pub fn new(spawn: (Vec2, f32), kit: usize) -> Self {
        let mut p = Player {
            pos: spawn.0,
            ang: spawn.1,
            hp: 0,
            kit,
            state: State::Idle,
            frame: 0,
            age: 0,
            cd: [0.0; 4],
            cast_ticks: 0,
            lock: 0,
            combo: 0,
            charges: WIND_CHARGES,
            dashes: DASH_CHARGES,
            dash_gap: 0.0,
            dash_cd: 0.0,
            untargetable: false,
            target: spawn.0,
            cast: None,
            knock: Vec2::ZERO,
            timer: 0,
            finished: false,
            combo_timer: 0.0,
            spawn,
        };
        p.respawn();
        p
    }

    pub fn kit(&self) -> &'static Kit {
        kit(self.kit)
    }

    pub fn max_hp(&self) -> i32 {
        self.kit().hp
    }

    pub fn respawn(&mut self) {
        self.pos = self.spawn.0;
        self.ang = self.spawn.1;
        self.knock = Vec2::ZERO;
        self.hp = self.max_hp();
        self.cd = [0.0; 4];
        self.lock = 0;
        self.combo = 0;
        self.charges = WIND_CHARGES;
        self.dashes = DASH_CHARGES;
        self.dash_gap = 0.0;
        self.dash_cd = 0.0;
        self.untargetable = false;
        self.cast = None;
        self.reset_to(State::Idle);
    }

    pub fn facing(&self) -> Vec2 {
        Vec2::from_angle(self.ang)
    }

    pub fn dir8(&self) -> usize {
        dir8(self.ang)
    }

    pub fn frame_data(&self) -> &'static Frame {
        &self.state.anim().frames[self.frame]
    }

    pub fn anim_ticks(&self) -> u32 {
        if self.state == State::Attack {
            self.cast_ticks
        } else {
            self.state.anim().frames.iter().map(|f| f.ticks).sum()
        }
    }

    pub fn hurtbox(&self) -> Option<f32> {
        if self.untargetable {
            None
        } else {
            self.frame_data().hurt
        }
    }

    pub fn ready(&self, slot: usize) -> bool {
        if self.kit().skills[slot].charged() {
            self.charges > 0
        } else {
            self.cd[slot] <= 0.0
        }
    }

    pub fn can_dash(&self) -> bool {
        self.dashes > 0 && self.dash_gap <= 0.0
    }

    pub fn cooldown_frac(&self, slot: usize) -> f32 {
        let skill = self.kit().skills[slot];
        if skill.charged() && self.charges >= WIND_CHARGES {
            return 0.0;
        }
        self.cd[slot] / skill.cooldown()
    }

    pub fn take_hit(&mut self, dir: Vec2, dmg: i32) {
        self.reset_to(State::Hit);
        self.lock = 0;
        self.knock = dir * 3.0;
        self.hp -= dmg;
    }

    pub fn clamp(&mut self) {
        let m = RADIUS + 2.0;
        self.pos = self.pos.clamp(vec2(m, m), ARENA - m);
    }

    pub fn step(&mut self, input: Input) {
        self.tick_timers();
        if self.lock > 0 {
            self.lock -= 1;
            self.drift();
            self.advance();
            return;
        }
        let ready = input.slot.filter(|s| self.ready(*s));
        let quick = self.state == State::Attack && ready.is_some() && self.age >= QUICK_CAST_AGE;
        match self.state {
            State::Hit | State::Attack if !quick => {
                if self.finished {
                    self.set_state(State::Idle);
                }
            }
            State::Roll => {
                self.pos += self.facing() * ROLL_SPEED * DT;
                if self.finished {
                    self.set_state(State::Idle);
                }
            }
            _ => self.act(input, ready),
        }
        self.drift();
        self.advance();
    }

    fn act(&mut self, input: Input, ready: Option<usize>) {
        let dir = input.dir.normalize_or_zero();
        if input.dodge && self.can_dash() {
            self.dashes -= 1;
            self.dash_gap = DASH_GAP;
            if self.dash_cd <= 0.0 {
                self.dash_cd = DASH_RECHARGE;
            }
            self.face(input.aim);
            self.set_state(State::Roll);
        } else if let Some(slot) = ready {
            self.begin_cast(slot, input.aim);
        } else if dir != Vec2::ZERO {
            self.pos += dir * SPEED * DT;
            self.ang = dir.y.atan2(dir.x);
            self.set_state(State::Run);
        } else {
            self.set_state(State::Idle);
        }
    }

    fn face(&mut self, at: Vec2) {
        let to = at - self.pos;
        if to.length() > 0.5 {
            self.ang = to.y.atan2(to.x);
        }
    }

    fn begin_cast(&mut self, slot: usize, aim: Vec2) {
        let skill = self.kit().skills[slot];
        self.face(aim);
        self.target = aim;
        self.cast_ticks = skill.cast_ticks();
        self.lock = skill.lock_ticks();
        match skill {
            Skill::WindStep => {
                self.charges -= 1;
                if self.cd[slot] <= 0.0 {
                    self.cd[slot] = skill.cooldown();
                }
            }
            Skill::Cut => {
                self.combo = self.combo % 3 + 1;
                self.combo_timer = COMBO_WINDOW;
                self.cd[slot] = if self.combo == 3 { COMBO_FINISHER_CD } else { skill.cooldown() };
            }
            _ => self.cd[slot] = skill.cooldown(),
        }
        self.cast = Some(skill);
        self.reset_to(State::Attack);
    }

    fn tick_timers(&mut self) {
        self.cd.iter_mut().for_each(|c| *c = (*c - DT).max(0.0));
        if let Some(slot) = self.kit().skills.iter().position(|s| s.charged()) {
            if self.charges < WIND_CHARGES && self.cd[slot] <= 0.0 {
                self.charges += 1;
                if self.charges < WIND_CHARGES {
                    self.cd[slot] = Skill::WindStep.cooldown();
                }
            }
        }
        self.combo_timer = (self.combo_timer - DT).max(0.0);
        self.dash_gap = (self.dash_gap - DT).max(0.0);
        if self.dashes < DASH_CHARGES {
            self.dash_cd = (self.dash_cd - DT).max(0.0);
            if self.dash_cd <= 0.0 {
                self.dashes += 1;
                if self.dashes < DASH_CHARGES {
                    self.dash_cd = DASH_RECHARGE;
                }
            }
        }
        if self.combo_timer <= 0.0 {
            self.combo = 0;
        }
    }

    fn drift(&mut self) {
        self.pos += self.knock;
        self.knock *= KNOCK_DECAY;
        self.clamp();
    }

    fn reset_to(&mut self, s: State) {
        self.state = s;
        self.frame = 0;
        self.timer = 0;
        self.finished = false;
        self.age = 0;
    }

    fn set_state(&mut self, s: State) {
        if self.state != s {
            self.reset_to(s);
        }
    }

    fn advance(&mut self) {
        let anim = self.state.anim();
        self.age += 1;
        self.timer += 1;
        if self.timer >= anim.frames[self.frame].ticks {
            self.timer = 0;
            if self.frame + 1 < anim.frames.len() {
                self.frame += 1;
            } else if anim.looping {
                self.frame = 0;
            } else {
                self.finished = true;
            }
        }
    }
}

pub fn dir8(ang: f32) -> usize {
    ((ang / (PI / 4.0)).round() as i32).rem_euclid(8) as usize
}
