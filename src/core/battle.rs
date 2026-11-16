use std::f32::consts::PI;

use macroquad::math::{vec2, Vec2};

use crate::core::{
    event::{Element, Event},
    magic::Spells,
    player::{Input, Player},
    sword::Blades,
    hex::Hexes,
    Skill, RADIUS,
};

pub const SPAWNS: [(Vec2, f32); 2] = [(vec2(140.0, 120.0), 0.0), (vec2(340.0, 120.0), PI)];

pub struct Battle {
    pub players: [Player; 2],
    pub spells: Spells,
    pub blades: Blades,
    pub hexes: Hexes,
    pub deaths: [u32; 2],
    events: Vec<Event>,
}

impl Battle {
    pub fn new(kits: [usize; 2], seat: usize) -> Self {
        let mine = SPAWNS[seat];
        let theirs = SPAWNS[1 - seat];
        Battle {
            players: [Player::new(mine, kits[0]), Player::new(theirs, kits[1])],
            spells: Spells::default(),
            blades: Blades::default(),
            hexes: Hexes::default(),
            deaths: [0; 2],
            events: Vec::new(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn tick(&mut self, inputs: [Input; 2]) {
        for (i, input) in inputs.into_iter().enumerate() {
            self.players[i].step(input);
            if let Some(skill) = self.players[i].cast.take() {
                self.cast(i, skill);
            }
        }
        self.push_apart();
        self.spells.update(&mut self.players, &mut self.events);
        self.blades.update(&mut self.players, &mut self.events);
        self.hexes.update(&mut self.players, &self.spells.wards, &mut self.events);
        for seat in 0..2 {
            if self.players[seat].hp <= 0 {
                self.players[seat].respawn();
                self.deaths[seat] += 1;
            }
        }
    }

    fn cast(&mut self, i: usize, skill: Skill) {
        match skill {
            Skill::Needle | Skill::Ray | Skill::Ward | Skill::Judgement => {
                self.spells.cast(&mut self.players, i, skill, &mut self.events)
            }
            Skill::Cut | Skill::Swallow | Skill::WindStep | Skill::Formation => {
                self.blades.cast(&mut self.players, i, skill, &mut self.events)
            }
            Skill::Hex | Skill::Potion | Skill::Broom | Skill::Cauldron => {
                self.hexes.cast(&mut self.players, i, skill, &mut self.events)
            }
        }
    }

    fn push_apart(&mut self) {
        let d = self.players[1].pos - self.players[0].pos;
        let len = d.length().max(0.01);
        let min = RADIUS * 2.0;
        if len < min {
            let n = d / len * (min - len) / 2.0;
            self.players[0].pos -= n;
            self.players[1].pos += n;
            self.players.iter_mut().for_each(Player::clamp);
        }
    }
}

pub(crate) fn strike(players: &mut [Player; 2], target: usize, dir: Vec2, dmg: i32, element: Element, big: bool, events: &mut Vec<Event>) -> bool {
    if players[target].hurtbox().is_none() {
        return false;
    }
    players[target].take_hit(dir, dmg);
    events.push(Event::Hit { pos: players[target].pos, element, dmg, big });
    true
}
