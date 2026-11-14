use crate::anim::*;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub const ARENA: f32 = 200.0;
pub const RADIUS: f32 = 6.0;
pub const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 72.0;
const MAX_HP: i32 = 100;

pub struct Controls {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub attack: KeyCode,
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub dir: Vec2,
    pub attack: bool,
}

impl Controls {
    pub fn read(&self) -> Input {
        let axis = |neg: KeyCode, pos: KeyCode| is_key_down(pos) as i32 as f32 - is_key_down(neg) as i32 as f32;
        Input {
            dir: vec2(axis(self.left, self.right), axis(self.up, self.down)),
            attack: is_key_pressed(self.attack),
        }
    }
}

pub struct Player {
    pub pos: Vec2,
    pub ang: f32,
    knock: Vec2,
    pub hp: i32,
    pub state: State,
    pub frame: usize,
    timer: u32,
    finished: bool,
    pub has_hit: bool,
    pub color: Color,
    pub controls: Controls,
    spawn: (Vec2, f32),
}

impl Player {
    pub fn new(pos: Vec2, ang: f32, color: Color, controls: Controls) -> Self {
        Player {
            pos,
            ang,
            knock: Vec2::ZERO,
            hp: MAX_HP,
            state: State::Idle,
            frame: 0,
            timer: 0,
            finished: false,
            has_hit: false,
            color,
            controls,
            spawn: (pos, ang),
        }
    }

    pub fn respawn(&mut self) {
        self.pos = self.spawn.0;
        self.ang = self.spawn.1;
        self.knock = Vec2::ZERO;
        self.hp = MAX_HP;
        self.reset_to(State::Idle);
    }

    fn reset_to(&mut self, s: State) {
        self.state = s;
        self.frame = 0;
        self.timer = 0;
        self.finished = false;
        self.has_hit = false;
    }

    fn set_state(&mut self, s: State) {
        if self.state != s {
            self.reset_to(s);
        }
    }

    pub fn facing(&self) -> Vec2 {
        Vec2::from_angle(self.ang)
    }

    pub fn dir8(&self) -> i32 {
        ((self.ang / (PI / 4.0)).round() as i32).rem_euclid(8)
    }

    pub fn step(&mut self, input: Input, anims: &Anims) {
        match self.state {
            State::Hit | State::Attack => {
                if self.finished {
                    self.set_state(State::Idle);
                }
            }
            _ => {
                let dir = input.dir.normalize_or_zero();
                if input.attack {
                    self.set_state(State::Attack);
                } else if dir != Vec2::ZERO {
                    self.pos += dir * SPEED * DT;
                    self.ang = dir.y.atan2(dir.x);
                    self.set_state(State::Run);
                } else {
                    self.set_state(State::Idle);
                }
            }
        }
        self.pos += self.knock;
        self.knock *= 0.85;
        self.clamp();
        self.advance(anims);
    }

    pub fn clamp(&mut self) {
        let m = RADIUS + 2.0;
        self.pos = self.pos.clamp(vec2(m, m), vec2(ARENA - m, ARENA - m));
    }

    fn advance(&mut self, anims: &Anims) {
        let anim = anims.get(self.state);
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

    pub fn current<'a>(&self, anims: &'a Anims) -> &'a Frame {
        &anims.get(self.state).frames[self.frame]
    }

    pub fn hits(&self, shape: &Shape, target: Vec2) -> bool {
        match *shape {
            Shape::Circle { fwd, side, r } => {
                let f = self.facing();
                let c = self.pos + f * fwd + f.perp() * side;
                c.distance(target) < r + RADIUS
            }
            Shape::Cone { r, half } => {
                let to = target - self.pos;
                let da = self.facing().angle_between(to).abs();
                to.length() < r + RADIUS && da < half
            }
        }
    }

    pub fn take_hit(&mut self, dir: Vec2, dmg: i32) {
        self.reset_to(State::Hit);
        self.knock = dir * 3.0;
        self.hp -= dmg;
    }
}
