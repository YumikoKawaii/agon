use crate::anim::*;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub const ARENA: f32 = 400.0;
pub const RADIUS: f32 = 6.0;
pub const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 72.0;
const ROLL_SPEED: f32 = 160.0;
const MAX_HP: i32 = 100;

const BLINK: f32 = 70.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Skill {
    Bolt,
    Orb,
    Blink,
    Star,
}

pub const SKILLS: [Skill; 4] = [Skill::Bolt, Skill::Orb, Skill::Blink, Skill::Star];
pub const COOLDOWN: [f32; 4] = [0.35, 3.0, 4.0, 10.0];

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub dir: Vec2,
    pub aim: Vec2,
    pub skill: Option<Skill>,
    pub dodge: bool,
}

pub fn read_input(aim: Vec2) -> Input {
    let axis = |neg: KeyCode, pos: KeyCode| is_key_down(pos) as i32 as f32 - is_key_down(neg) as i32 as f32;
    let pressed = [
        is_mouse_button_pressed(MouseButton::Left),
        is_key_pressed(KeyCode::Q),
        is_key_pressed(KeyCode::E),
        is_key_pressed(KeyCode::R),
    ];
    Input {
        dir: vec2(axis(KeyCode::A, KeyCode::D), axis(KeyCode::W, KeyCode::S)),
        aim,
        skill: (0..4).find(|i| pressed[*i]).map(|i| SKILLS[i]),
        dodge: is_mouse_button_pressed(MouseButton::Right),
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
    pub age: u32,
    pub cd: [f32; 4],
    pub cast: Option<Skill>,
    pub blink_from: Option<Vec2>,
    pub has_hit: bool,
    pub color: Color,
    pub tint: Color,
    pub dummy: bool,
    pub target: Vec2,
    spawn: (Vec2, f32),
}

impl Player {
    pub fn new(pos: Vec2, ang: f32, color: Color, dummy: bool) -> Self {
        Player {
            pos,
            ang,
            knock: Vec2::ZERO,
            hp: MAX_HP,
            state: State::Idle,
            frame: 0,
            timer: 0,
            finished: false,
            age: 0,
            cd: [0.0; 4],
            cast: None,
            blink_from: None,
            has_hit: false,
            tint: if dummy { Color::new(0.75, 0.75, 0.8, 1.0) } else { WHITE },
            color,
            dummy,
            target: pos,
            spawn: (pos, ang),
        }
    }

    pub fn respawn(&mut self) {
        self.pos = self.spawn.0;
        self.ang = self.spawn.1;
        self.knock = Vec2::ZERO;
        self.hp = MAX_HP;
        self.cd = [0.0; 4];
        self.reset_to(State::Idle);
    }

    fn reset_to(&mut self, s: State) {
        self.state = s;
        self.frame = 0;
        self.timer = 0;
        self.finished = false;
        self.age = 0;
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
        self.cd.iter_mut().for_each(|c| *c = (*c - DT).max(0.0));
        let ready = input.skill.filter(|k| self.cd[*k as usize] <= 0.0);
        match self.state {
            State::Hit | State::Attack => {
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
            _ => {
                let dir = input.dir.normalize_or_zero();
                if input.dodge {
                    let to = input.aim - self.pos;
                    if to.length() > 0.5 {
                        self.ang = to.y.atan2(to.x);
                    }
                    self.set_state(State::Roll);
                } else if let Some(k) = ready {
                    let to = input.aim - self.pos;
                    if to.length() > 0.5 {
                        self.ang = to.y.atan2(to.x);
                    }
                    self.target = input.aim;
                    if k == Skill::Blink {
                        self.blink_from = Some(self.pos);
                        self.pos += self.facing() * to.length().min(BLINK);
                        self.clamp();
                    }
                    self.cd[k as usize] = COOLDOWN[k as usize];
                    self.cast = Some(k);
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

    pub fn current<'a>(&self, anims: &'a Anims) -> &'a Frame {
        &anims.get(self.state).frames[self.frame]
    }

    pub fn place(&self, c: &Circ) -> Vec2 {
        let f = self.facing();
        self.pos + f * c.fwd + f.perp() * c.side
    }

    pub fn hurtboxes(&self, anims: &Anims) -> Vec<(Vec2, f32)> {
        self.current(anims)
            .hurt
            .iter()
            .map(|c| (self.place(c), c.r))
            .collect()
    }

    pub fn hits(&self, shape: &Shape, target: Vec2, target_r: f32) -> bool {
        match shape {
            Shape::Circle(c) => self.place(c).distance(target) < c.r + target_r,
            Shape::Cone { r, half } => {
                let to = target - self.pos;
                let da = self.facing().angle_between(to).abs();
                to.length() < r + target_r && da < *half
            }
        }
    }

    pub fn take_hit(&mut self, dir: Vec2, dmg: i32) {
        self.reset_to(State::Hit);
        self.knock = dir * 3.0;
        self.hp -= dmg;
    }
}
