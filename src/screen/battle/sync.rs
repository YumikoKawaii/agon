use macroquad::math::Vec2;

use crate::{
    core::{Input, Player},
    net::{
        link::{Control, Link, LinkState},
        Packet,
    },
};

const RESULT_HOLD: f32 = 3.0;
const SNAP_BLEND: f32 = 0.5;

pub struct Sync {
    link: Link,
    pub seat: usize,
    pub foe: String,
    pub started: bool,
    result: Option<(usize, f32)>,
    tick: u32,
    cast_seq: u16,
    cast_slot: u8,
    dodge_seq: u8,
    remote: Option<Packet>,
    seen_cast: u16,
    seen_dodge: u8,
    pub foe_deaths: u32,
}

impl Sync {
    pub fn new(link: Link, seat: usize, foe: String) -> Self {
        Sync { link, seat, foe, started: false, result: None, tick: 0, cast_seq: 0, cast_slot: 0, dodge_seq: 0, remote: None, seen_cast: 0, seen_dodge: 0, foe_deaths: 0 }
    }

    pub fn live(&self) -> bool {
        self.started && self.result.is_none()
    }

    pub fn finished(&mut self, dt: f32) -> bool {
        for c in self.link.control() {
            match c {
                Control::Start => self.started = true,
                Control::End { winner } => self.result = Some((winner, RESULT_HOLD)),
            }
        }
        match self.result.as_mut() {
            Some((_, t)) => {
                *t -= dt;
                *t <= 0.0
            }
            None => self.link.state() == LinkState::Closed,
        }
    }

    pub fn outcome(&self) -> Option<bool> {
        self.result.map(|(w, _)| w == self.seat)
    }

    pub fn banner(&self) -> String {
        match self.result {
            Some((w, _)) if w == self.seat => "VICTORY".to_string(),
            Some(_) => format!("{} wins", self.foe),
            None if self.started => String::new(),
            None if self.link.state() == LinkState::Connecting => "connecting...".to_string(),
            None => "waiting for opponent...".to_string(),
        }
    }

    pub fn pull(&mut self) {
        for p in self.link.recv() {
            if self.remote.is_some_and(|r| p.tick <= r.tick) {
                continue;
            }
            if self.remote.is_none() {
                self.seen_cast = p.cast_seq;
                self.seen_dodge = p.dodge_seq;
            }
            self.foe_deaths = self.foe_deaths.max(p.deaths);
            self.remote = Some(p);
        }
    }

    pub fn remote_input(&mut self) -> Input {
        let Some(p) = self.remote else { return Input::default() };
        let mut input = Input { dir: Vec2::from(p.dir), aim: Vec2::from(p.aim), slot: None, dodge: false };
        if p.cast_seq != self.seen_cast {
            self.seen_cast = p.cast_seq;
            input.slot = Some(p.cast_slot as usize);
        }
        if p.dodge_seq != self.seen_dodge {
            self.seen_dodge = p.dodge_seq;
            input.dodge = true;
        }
        input
    }

    pub fn correct(&self, foe: &mut Player) {
        if let Some(p) = self.remote {
            foe.pos = foe.pos.lerp(Vec2::from(p.pos), SNAP_BLEND);
            foe.ang = p.ang;
            foe.hp = p.hp;
            foe.effects.sync(p.fx_ticks, p.fx_power, p.fx_stacks);
        }
    }

    pub fn push(&mut self, input: &Input, me: &Player, deaths: u32) {
        self.tick += 1;
        if let Some(s) = input.slot {
            self.cast_seq = self.cast_seq.wrapping_add(1);
            self.cast_slot = s as u8;
        }
        if input.dodge {
            self.dodge_seq = self.dodge_seq.wrapping_add(1);
        }
        let (fx_ticks, fx_power, fx_stacks) = me.effects.wire();
        self.link.send(&Packet {
            tick: self.tick,
            dir: input.dir.into(),
            aim: input.aim.into(),
            pos: me.pos.into(),
            ang: me.ang,
            hp: me.hp,
            deaths,
            cast_seq: self.cast_seq,
            cast_slot: self.cast_slot,
            dodge_seq: self.dodge_seq,
            fx_ticks,
            fx_power,
            fx_stacks,
        });
    }
}
