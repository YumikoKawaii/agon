use crate::anim::*;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub const ARENA: f32 = 400.0;
pub const RADIUS: f32 = 6.0;
pub const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 72.0;
const ROLL_SPEED: f32 = 160.0;
pub const WIND_CHARGES: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Skill {
    Needle,
    Ray,
    Ward,
    Judgement,
    Cut,
    Swallow,
    WindStep,
    Formation,
}

impl Skill {
    pub fn cooldown(self) -> f32 {
        match self {
            Skill::Needle => 0.3,
            Skill::Ray => 4.0,
            Skill::Ward => 6.0,
            Skill::Judgement => 12.0,
            Skill::Cut => 0.22,
            Skill::Swallow => 4.0,
            Skill::WindStep => 3.0,
            Skill::Formation => 14.0,
        }
    }

    fn cast_ticks(self) -> u32 {
        match self {
            Skill::Needle => 18,
            Skill::Ray => 36,
            Skill::Ward => 20,
            Skill::Judgement => 40,
            Skill::Cut => 13,
            Skill::Swallow => 14,
            Skill::WindStep => 10,
            Skill::Formation => 30,
        }
    }

    fn lock_ticks(self) -> u32 {
        match self {
            Skill::Ray => 30,
            Skill::Judgement => 24,
            Skill::Formation => 26,
            Skill::Swallow => 12,
            _ => 0,
        }
    }
}

pub struct Kit {
    pub name: &'static str,
    pub skills: [Skill; 4],
    pub hp: i32,
    pub hint: &'static str,
}

pub const KITS: [Kit; 3] = [
    Kit {
        name: "Elf Mage",
        skills: [Skill::Needle, Skill::Ray, Skill::Ward, Skill::Judgement],
        hp: 100,
        hint: "LMB needle   Q piercing ray   E ward shell   R judgement circle",
    },
    Kit {
        name: "Witch",
        skills: [Skill::Needle, Skill::Ray, Skill::Ward, Skill::Judgement],
        hp: 100,
        hint: "LMB needle   Q piercing ray   E ward shell   R judgement circle",
    },
    Kit {
        name: "Sword Cultivator",
        skills: [Skill::Cut, Skill::Swallow, Skill::WindStep, Skill::Formation],
        hp: 80,
        hint: "LMB swift cut (x3)   Q swallow strike (hit: +1 E charge)   E wind step (2 charges)   R sword formation",
    },
];

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub dir: Vec2,
    pub aim: Vec2,
    pub slot: Option<usize>,
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
        slot: (0..4).find(|i| pressed[*i]),
        dodge: is_mouse_button_pressed(MouseButton::Right),
    }
}

pub struct Player {
    pub pos: Vec2,
    pub ang: f32,
    knock: Vec2,
    pub hp: i32,
    pub max_hp: i32,
    pub kit: usize,
    pub state: State,
    pub frame: usize,
    timer: u32,
    finished: bool,
    pub age: u32,
    pub cd: [f32; 4],
    pub cast: Option<Skill>,
    pub cast_ticks: u32,
    pub lock: u32,
    pub combo: u32,
    combo_timer: f32,
    pub charges: u32,
    pub recast: bool,
    pub untargetable: bool,
    pub ghosts: Vec<(Vec2, f32, i32)>,
    pub has_hit: bool,
    pub color: Color,
    pub tint: Color,
    pub dummy: bool,
    pub target: Vec2,
    spawn: (Vec2, f32),
}

impl Player {
    pub fn new(pos: Vec2, ang: f32, color: Color, dummy: bool, kit: usize) -> Self {
        Player {
            pos,
            ang,
            knock: Vec2::ZERO,
            hp: KITS[kit].hp,
            max_hp: KITS[kit].hp,
            kit,
            state: State::Idle,
            frame: 0,
            timer: 0,
            finished: false,
            age: 0,
            cd: [0.0; 4],
            cast: None,
            cast_ticks: 18,
            lock: 0,
            combo: 0,
            combo_timer: 0.0,
            charges: WIND_CHARGES,
            recast: false,
            untargetable: false,
            ghosts: Vec::new(),
            has_hit: false,
            tint: WHITE,
            color,
            dummy,
            target: pos,
            spawn: (pos, ang),
        }
    }

    pub fn skills(&self) -> [Skill; 4] {
        KITS[self.kit].skills
    }

    pub fn set_kit(&mut self, kit: usize) {
        self.kit = kit;
        self.max_hp = KITS[kit].hp;
        self.respawn();
    }

    pub fn set_spawn(&mut self, pos: Vec2, ang: f32) {
        self.spawn = (pos, ang);
    }

    pub fn respawn(&mut self) {
        self.pos = self.spawn.0;
        self.ang = self.spawn.1;
        self.knock = Vec2::ZERO;
        self.hp = self.max_hp;
        self.cd = [0.0; 4];
        self.lock = 0;
        self.combo = 0;
        self.charges = WIND_CHARGES;
        self.recast = false;
        self.untargetable = false;
        self.ghosts.clear();
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

    pub fn ghost(&mut self) {
        let d = self.dir8();
        self.ghosts.push((self.pos, 0.3, d));
    }

    fn ready(&self, slot: usize) -> bool {
        match self.skills()[slot] {
            Skill::WindStep => self.charges > 0,
            _ => self.cd[slot] <= 0.0,
        }
    }

    fn tick_timers(&mut self) {
        self.cd.iter_mut().for_each(|c| *c = (*c - DT).max(0.0));
        let skills = self.skills();
        if let Some(s) = skills.iter().position(|k| *k == Skill::WindStep) {
            if self.charges < WIND_CHARGES && self.cd[s] <= 0.0 {
                self.charges += 1;
                if self.charges < WIND_CHARGES {
                    self.cd[s] = Skill::WindStep.cooldown();
                }
            }
        }
        self.combo_timer = (self.combo_timer - DT).max(0.0);
        if self.combo_timer <= 0.0 {
            self.combo = 0;
        }
        self.ghosts.retain_mut(|g| {
            g.1 -= DT;
            g.1 > 0.0
        });
    }

    fn begin_cast(&mut self, slot: usize, aim: Vec2) {
        let k = self.skills()[slot];
        let to = aim - self.pos;
        if to.length() > 0.5 {
            self.ang = to.y.atan2(to.x);
        }
        self.target = aim;
        self.cast_ticks = k.cast_ticks();
        self.lock = k.lock_ticks();
        match k {
            Skill::WindStep => {
                self.charges -= 1;
                if self.cd[slot] <= 0.0 {
                    self.cd[slot] = k.cooldown();
                }
            }
            Skill::Cut => {
                self.combo = self.combo % 3 + 1;
                self.combo_timer = 0.7;
                self.cd[slot] = if self.combo == 3 { 0.45 } else { k.cooldown() };
            }
            _ => self.cd[slot] = k.cooldown(),
        }
        self.cast = Some(k);
        self.reset_to(State::Attack);
    }

    pub fn step(&mut self, input: Input, anims: &Anims) {
        self.tick_timers();
        if self.lock > 0 {
            self.lock -= 1;
            self.pos += self.knock;
            self.knock *= 0.85;
            self.clamp();
            self.advance(anims);
            return;
        }
        let ready = input.slot.filter(|s| self.ready(*s));
        let quick = self.state == State::Attack && ready.is_some() && self.age >= 6;
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
            _ => {
                let dir = input.dir.normalize_or_zero();
                if input.dodge {
                    let to = input.aim - self.pos;
                    if to.length() > 0.5 {
                        self.ang = to.y.atan2(to.x);
                    }
                    self.set_state(State::Roll);
                } else if let Some(s) = ready {
                    self.begin_cast(s, input.aim);
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
        if self.untargetable {
            return Vec::new();
        }
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
        self.lock = 0;
        self.knock = dir * 3.0;
        self.hp -= dmg;
    }
}
