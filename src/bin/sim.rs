#[path = "../core/mod.rs"]
mod core;

use core::{
    bot::{Bot, Level, LEVELS},
    Battle, DEATHS_TO_WIN, DT, KITS,
};
use cfg::{Ai, Config};
use nanoserde::DeJson;

mod cfg {
    use nanoserde::DeJson;

    #[derive(DeJson, Clone)]
    pub struct Ai {
        pub kit: String,
        pub level: String,
        pub reaction_ticks: Option<usize>,
        pub aim_error: Option<f32>,
        pub lead: Option<f32>,
        pub dodge_read: Option<f32>,
        pub hesitate: Option<f32>,
        pub aggression: Option<f32>,
    }

    #[derive(DeJson)]
    pub struct Config {
        pub matches: usize,
        pub max_seconds: f32,
        pub seed: u64,
        pub a: Ai,
        pub b: Ai,
    }
}

#[derive(Default)]
struct Tally {
    wins: [usize; 2],
    draws: usize,
    kills: [u32; 2],
    dmg: [i64; 2],
    ticks: u64,
}

impl Ai {
    fn level(&self) -> Level {
        let base = LEVELS.iter().find(|(n, _)| n.eq_ignore_ascii_case(&self.level)).unwrap_or_else(|| panic!("unknown level {}", self.level)).1;
        Level {
            reaction_ticks: self.reaction_ticks.unwrap_or(base.reaction_ticks),
            aim_error: self.aim_error.unwrap_or(base.aim_error),
            lead: self.lead.unwrap_or(base.lead),
            dodge_read: self.dodge_read.unwrap_or(base.dodge_read),
            hesitate: self.hesitate.unwrap_or(base.hesitate),
            aggression: self.aggression.unwrap_or(base.aggression),
        }
    }

    fn kits(&self) -> Vec<usize> {
        if self.kit.eq_ignore_ascii_case("all") {
            return (0..KITS.len()).collect();
        }
        vec![KITS.iter().position(|k| k.name.eq_ignore_ascii_case(&self.kit)).unwrap_or_else(|| panic!("unknown kit {}", self.kit))]
    }
}

fn duel(kits: [usize; 2], levels: [Level; 2], max_ticks: u64, t: &mut Tally) {
    let mut b = Battle::new(kits, 0);
    let mut bots = [Bot::new(levels[0], 0), Bot::new(levels[1], 1)];
    let mut tick = 0;
    while tick < max_ticks {
        let hp = [b.players[0].hp, b.players[1].hp];
        let deaths = b.deaths;
        let inputs = [bots[0].think(&b), bots[1].think(&b)];
        b.tick(inputs);
        b.drain_events();
        tick += 1;
        for s in 0..2 {
            let lost = if b.deaths[s] > deaths[s] { hp[s] } else { (hp[s] - b.players[s].hp).max(0) };
            t.dmg[1 - s] += lost as i64;
        }
        if b.deaths.iter().any(|d| *d >= DEATHS_TO_WIN) {
            break;
        }
    }
    t.ticks += tick;
    t.kills[0] += b.deaths[1];
    t.kills[1] += b.deaths[0];
    match (b.deaths[1] >= DEATHS_TO_WIN, b.deaths[0] >= DEATHS_TO_WIN) {
        (true, false) => t.wins[0] += 1,
        (false, true) => t.wins[1] += 1,
        _ => t.draws += 1,
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "sim.json".to_owned());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let cfg = Config::deserialize_json(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    macroquad::rand::srand(cfg.seed);
    let levels = [cfg.a.level(), cfg.b.level()];
    let max_ticks = (cfg.max_seconds / DT) as u64;
    println!("A: {} {:?}", cfg.a.level, fmt(levels[0]));
    println!("B: {} {:?}", cfg.b.level, fmt(levels[1]));
    println!("{} matches per pairing, first to {} kills, {}s cap, seed {}\n", cfg.matches, DEATHS_TO_WIN, cfg.max_seconds, cfg.seed);
    println!("{:<18} {:<18} {:>6} {:>6} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7}", "A kit", "B kit", "A win", "B win", "draw", "A win%", "A kills", "B kills", "A dps", "B dps");
    for ka in cfg.a.kits() {
        for kb in cfg.b.kits() {
            let mut t = Tally::default();
            for _ in 0..cfg.matches {
                duel([ka, kb], levels, max_ticks, &mut t);
            }
            let secs = (t.ticks as f32 * DT).max(DT);
            let n = cfg.matches as f32;
            println!(
                "{:<18} {:<18} {:>6} {:>6} {:>6} {:>6.1}% {:>7.2} {:>7.2} {:>7.1} {:>7.1}",
                KITS[ka].name, KITS[kb].name, t.wins[0], t.wins[1], t.draws,
                t.wins[0] as f32 / n * 100.0, t.kills[0] as f32 / n, t.kills[1] as f32 / n,
                t.dmg[0] as f32 / secs, t.dmg[1] as f32 / secs,
            );
        }
    }
}

fn fmt(l: Level) -> (usize, f32, f32, f32, f32, f32) {
    (l.reaction_ticks, l.aim_error, l.lead, l.dodge_read, l.hesitate, l.aggression)
}
