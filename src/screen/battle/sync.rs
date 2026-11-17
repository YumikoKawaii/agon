use std::collections::VecDeque;

use macroquad::math::Vec2;

use crate::{
    core::{self, Input},
    net::{
        link::{Control, Link, LinkState},
        Packet,
    },
};

const RESULT_HOLD: f32 = 3.0;
const SNAP_BLEND: f32 = 0.5;
const SLOTS: usize = 4;
const FOE: usize = 1;

pub struct Sync {
    link: Link,
    pub seat: usize,
    pub foe: String,
    pub started: bool,
    result: Option<(usize, f32)>,
    tick: u32,
    cast_seq: u16,
    cast_slot: u8,
    prev_slot: u8,
    remote: Option<Packet>,
    seen_cast: u16,
    casts: VecDeque<usize>,
    pub foe_deaths: u32,
    applied_deaths: u32,
}

impl Sync {
    pub fn new(link: Link, seat: usize, foe: String) -> Self {
        Sync {
            link,
            seat,
            foe,
            started: false,
            result: None,
            tick: 0,
            cast_seq: 0,
            cast_slot: 0,
            prev_slot: 0,
            remote: None,
            seen_cast: 0,
            casts: VecDeque::new(),
            foe_deaths: 0,
            applied_deaths: 0,
        }
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
                self.seen_cast = p.cast_seq.saturating_sub(1);
            }
            let missed = p.cast_seq.wrapping_sub(self.seen_cast);
            if missed >= 2 {
                self.casts.push_back(p.prev_slot as usize);
            }
            if missed >= 1 {
                self.casts.push_back(p.cast_slot as usize);
            }
            self.seen_cast = p.cast_seq;
            self.foe_deaths = self.foe_deaths.max(p.deaths);
            self.remote = Some(p);
        }
    }

    pub fn remote_input(&mut self) -> Input {
        let Some(p) = self.remote else { return Input::default() };
        let slot = self.casts.pop_front().filter(|s| *s < SLOTS);
        Input { dir: Vec2::from(p.dir), aim: Vec2::from(p.aim), slot }
    }

    pub fn correct(&mut self, battle: &mut core::Battle) {
        while self.applied_deaths < self.foe_deaths {
            self.applied_deaths += 1;
            battle.kill(FOE);
        }
        if let Some(p) = self.remote {
            let foe = &mut battle.players[FOE];
            foe.pos = foe.pos.lerp(Vec2::from(p.pos), SNAP_BLEND);
            foe.clamp();
            foe.ang = p.ang;
            foe.hp = p.hp;
            foe.effects.sync(p.fx_ticks, p.fx_power, p.fx_stacks);
        }
    }

    pub fn push(&mut self, input: &Input, battle: &core::Battle) {
        self.tick += 1;
        if let Some(s) = input.slot {
            self.cast_seq = self.cast_seq.wrapping_add(1);
            self.prev_slot = self.cast_slot;
            self.cast_slot = s as u8;
        }
        let me = &battle.players[0];
        let (fx_ticks, fx_power, fx_stacks) = me.effects.wire();
        self.link.send(&Packet {
            tick: self.tick,
            dir: input.dir.into(),
            aim: input.aim.into(),
            pos: me.pos.into(),
            ang: me.ang,
            hp: me.hp,
            deaths: battle.deaths[0],
            cast_seq: self.cast_seq,
            cast_slot: self.cast_slot,
            prev_slot: self.prev_slot,
            fx_ticks,
            fx_power,
            fx_stacks,
        });
    }
}
