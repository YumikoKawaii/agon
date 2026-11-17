use std::f32::consts::PI;

use macroquad::math::{vec2, Vec2};

use crate::core::{
    effect::{Apply, EffectKind},
    kit::Passive,
    player::GRACE_T,
    event::{Element, Event},
    magic::Spells,
    parasol::Parasols,
    fox::Foxes,
    seren::{Serens, OVER_MAX},
    player::{Input, Player},
    sword::Blades,
    hex::Hexes,
    Skill, DT, RADIUS,
};

pub const SPAWNS: [(Vec2, f32); 2] = [(vec2(140.0, 120.0), 0.0), (vec2(340.0, 120.0), PI)];

pub struct Zone {
    pub pos: Vec2,
    pub r: f32,
    pub t: f32,
    pub life: f32,
    pub owner: usize,
    pub apply: Apply,
}

impl Zone {
    pub fn fade(&self) -> f32 {
        (1.0 - self.t / self.life).clamp(0.0, 1.0)
    }
}

pub struct Battle {
    pub players: [Player; 2],
    pub spells: Spells,
    pub blades: Blades,
    pub hexes: Hexes,
    pub parasols: Parasols,
    pub foxes: Foxes,
    pub serens: Serens,
    pub zones: Vec<Zone>,
    pub deaths: [u32; 2],
    pub stay: bool,
    pub casted: [Option<usize>; 2],
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
            parasols: Parasols::default(),
            foxes: Foxes::default(),
            serens: Serens::default(),
            zones: Vec::new(),
            deaths: [0; 2],
            stay: false,
            casted: [None; 2],
            events: Vec::new(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn tick(&mut self, inputs: [Input; 2]) {
        for (i, input) in inputs.into_iter().enumerate() {
            self.players[i].step(input);
            self.casted[i] = None;
            if let Some(skill) = self.players[i].cast.take() {
                self.casted[i] = Some(self.players[i].cast_slot);
                self.cast(i, skill);
            }
        }
        self.push_apart();
        self.spells.update(&mut self.players, &mut self.events);
        self.blades.update(&mut self.players, &mut self.events);
        self.hexes.update(&mut self.players, &self.spells.wards, &mut self.zones, &mut self.events);
        self.parasols.update(&mut self.players, &self.spells.wards, &mut self.events);
        self.foxes.update(&mut self.players, &self.spells.wards, &mut self.events);
        self.serens.update(&mut self.players, &self.spells.wards, &mut self.events);
        self.update_zones();
        for seat in 0..2 {
            let pos = self.players[seat].pos;
            for d in self.players[seat].effects.tick() {
                if d.dmg > 0 {
                    self.players[seat].hp -= d.dmg;
                    self.events.push(Event::Dot { pos, kind: d.kind, dmg: d.dmg });
                }
            }
            if self.players[seat].hp <= 0 && !self.players[seat].puppet {
                self.kill(seat);
            }
        }
    }

    pub fn kill(&mut self, seat: usize) {
        let at = self.players[seat].pos;
        self.players[seat].respawn();
        if self.stay {
            self.players[seat].pos = at;
        }
        self.parasols.reset(seat);
        self.foxes.reset(seat);
        self.serens.reset(seat);
        self.deaths[seat] += 1;
    }

    fn update_zones(&mut self) {
        let players = &mut self.players;
        let events = &mut self.events;
        self.zones.retain_mut(|z| {
            z.t += DT;
            let t = 1 - z.owner;
            if players[t].hurtbox().is_some() && players[t].pos.distance(z.pos) < z.r + RADIUS {
                let fresh = !players[t].effects.on(z.apply.0);
                players[t].effects.add(z.apply);
                if fresh {
                    events.push(Event::Status { pos: players[t].pos, kind: z.apply.0 });
                }
            }
            z.t < z.life
        });
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
            Skill::Talisman | Skill::Parasol | Skill::SpiritStep | Skill::Storm | Skill::PetalDash | Skill::Blossom => {
                self.parasols.cast(&mut self.players, i, skill, &mut self.events)
            }
            Skill::FoxOrb | Skill::BlindLight | Skill::ShiningLight | Skill::Unpredictable | Skill::FoxClaw | Skill::FoxWisp | Skill::FoxSwipe | Skill::Vanish => {
                self.foxes.cast(&mut self.players, i, skill, &mut self.events)
            }
            Skill::ManaShot | Skill::AzureLance | Skill::GravityWell | Skill::Overdrive | Skill::RuneKnuckle | Skill::OverFists | Skill::RocketDash | Skill::Starbreaker => {
                self.serens.cast(&mut self.players, i, skill, &mut self.events)
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

pub struct Hit {
    pub dmg: i32,
    pub dir: Vec2,
    pub element: Element,
    pub big: bool,
    pub apply: &'static [Apply],
    pub knock: bool,
}

impl Hit {
    pub fn new(dmg: i32, dir: Vec2, element: Element, big: bool, apply: &'static [Apply]) -> Self {
        Hit { dmg, dir, element, big, apply, knock: false }
    }

    pub fn knock(mut self) -> Self {
        self.knock = true;
        self
    }
}

pub(crate) fn strike(players: &mut [Player; 2], target: usize, hit: Hit, events: &mut Vec<Event>) -> bool {
    if players[target].hurtbox().is_none() {
        return false;
    }
    let p = &mut players[target];
    let pos = p.pos;
    let raw = match p.kit().passive {
        Passive::Fox if !p.armed => (hit.dmg * 4 + 4) / 5,
        Passive::Overcharge if !p.armed => (hit.dmg * 17 + 19) / 20,
        _ => hit.dmg,
    };
    let (dmg, absorbed, marked) = p.effects.incoming(raw);
    p.take_hit(hit.dir, dmg, hit.knock || hit.apply.iter().any(|a| a.0 == EffectKind::Stun));
    if marked {
        events.push(Event::MarkPop { pos });
    }
    if absorbed > 0 {
        events.push(Event::Absorb { pos, amount: absorbed });
    }
    for a in hit.apply {
        let fresh = !p.effects.on(a.0);
        p.effects.add(*a);
        if fresh || a.0 == EffectKind::Bleed {
            events.push(Event::Status { pos, kind: a.0 });
        }
    }
    events.push(Event::Hit { pos, element: hit.element, dmg, big: hit.big || marked });
    passive(players, 1 - target, target, events);
    true
}

const HEX_MAX: u8 = 3;
const HEX_DMG: i32 = 3;
const HEX_ROOT: Apply = Apply(EffectKind::Root, 20, 0);
const INTENT_MAX: u8 = 4;

fn passive(players: &mut [Player; 2], me: usize, target: usize, events: &mut Vec<Event>) {
    match players[me].kit().passive {
        Passive::Grace => players[me].haste = GRACE_T,
        Passive::Hexed => {
            let t = &mut players[target];
            t.hex += 1;
            if t.hex >= HEX_MAX {
                t.hex = 0;
                t.hp -= HEX_DMG;
                t.effects.add(HEX_ROOT);
                events.push(Event::Status { pos: t.pos, kind: EffectKind::Root });
                events.push(Event::Hit { pos: t.pos, element: Element::Hex, dmg: HEX_DMG, big: true });
            }
        }
        Passive::Intent => {
            let p = &mut players[me];
            p.intent += 1;
            if p.intent >= INTENT_MAX {
                p.intent = 0;
                p.empower();
                events.push(Event::Status { pos: p.pos, kind: EffectKind::Vulnerable });
            }
        }
        Passive::Bond => {}
        Passive::Fox => {}
        Passive::Overcharge => {
            let p = &mut players[me];
            if p.over < OVER_MAX {
                p.over += 1;
                if p.over == OVER_MAX {
                    events.push(Event::SerenFull { pos: p.pos });
                }
            }
        }
    }
}

pub(crate) fn bless(players: &mut [Player; 2], seat: usize, a: Apply, events: &mut Vec<Event>) {
    players[seat].effects.add(a);
    events.push(Event::Status { pos: players[seat].pos, kind: a.0 });
}
