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
    Hex,
    Potion,
    Broom,
    Cauldron,
    Talisman,
    Parasol,
    Storm,
    SpiritStep,
    PetalDash,
    Blossom,
    FoxOrb,
    BlindLight,
    ShiningLight,
    Unpredictable,
    FoxClaw,
    FoxWisp,
    FoxSwipe,
    Vanish,
    ManaShot,
    AzureLance,
    GravityWell,
    Overdrive,
    RuneKnuckle,
    OverFists,
    RocketDash,
    Starbreaker,
}

impl Skill {
    pub fn unarmed(self) -> Skill {
        match self {
            Skill::PetalDash => Skill::SpiritStep,
            Skill::Blossom => Skill::Storm,
            Skill::FoxOrb => Skill::FoxClaw,
            Skill::BlindLight => Skill::FoxWisp,
            Skill::ShiningLight => Skill::FoxSwipe,
            Skill::Unpredictable => Skill::Vanish,
            Skill::ManaShot => Skill::RuneKnuckle,
            Skill::AzureLance => Skill::OverFists,
            Skill::GravityWell => Skill::RocketDash,
            Skill::Overdrive => Skill::Starbreaker,
            s => s,
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Skill::Needle => 1.2,
            Skill::Ray => 2.5,
            Skill::Ward => 4.0,
            Skill::Judgement => 9.0,
            Skill::Cut => 0.6,
            Skill::Swallow => 4.0,
            Skill::WindStep => 3.0,
            Skill::Formation => 8.0,
            Skill::Hex => 1.2,
            Skill::Potion => 3.0,
            Skill::Broom => 3.5,
            Skill::Cauldron => 10.0,
            Skill::Talisman => 1.1,
            Skill::Parasol => 1.5,
            Skill::Storm => 13.0,
            Skill::SpiritStep => 4.0,
            Skill::PetalDash => 4.0,
            Skill::Blossom => 10.0,
            Skill::FoxOrb => 0.55,
            Skill::BlindLight => 3.0,
            Skill::ShiningLight => 3.5,
            Skill::Unpredictable => 6.0,
            Skill::FoxClaw => 0.25,
            Skill::FoxWisp => 2.5,
            Skill::FoxSwipe => 2.5,
            Skill::Vanish => 6.0,
            Skill::ManaShot => 0.7,
            Skill::AzureLance => 1.2,
            Skill::GravityWell => 6.0,
            Skill::Overdrive => 7.0,
            Skill::RuneKnuckle => 0.5,
            Skill::OverFists => 7.0,
            Skill::RocketDash => 5.0,
            Skill::Starbreaker => 7.0,
        }
    }

    pub fn cast_ticks(self) -> u32 {
        match self {
            Skill::Needle => 18,
            Skill::Ray => 36,
            Skill::Ward => 20,
            Skill::Judgement => 40,
            Skill::Cut => 13,
            Skill::Swallow => 14,
            Skill::WindStep => 10,
            Skill::Formation => 30,
            Skill::Hex => 18,
            Skill::Potion => 20,
            Skill::Broom => 10,
            Skill::Cauldron => 36,
            Skill::Talisman => 16,
            Skill::Parasol => 14,
            Skill::Storm => 24,
            Skill::SpiritStep => 16,
            Skill::PetalDash => 10,
            Skill::Blossom => 20,
            Skill::FoxOrb => 16,
            Skill::BlindLight => 18,
            Skill::ShiningLight => 20,
            Skill::Unpredictable => 12,
            Skill::FoxClaw => 12,
            Skill::FoxWisp => 14,
            Skill::FoxSwipe => 24,
            Skill::Vanish => 10,
            Skill::ManaShot => 15,
            Skill::AzureLance => 10,
            Skill::GravityWell => 14,
            Skill::Overdrive => 40,
            Skill::RuneKnuckle => 12,
            Skill::OverFists => 10,
            Skill::RocketDash => 10,
            Skill::Starbreaker => 75,
        }
    }

    pub fn lock_ticks(self) -> u32 {
        match self {
            Skill::Ray => 30,
            Skill::Judgement => 24,
            Skill::Formation => 26,
            Skill::Swallow => 12,
            Skill::Cauldron => 20,
            Skill::SpiritStep => 8,
            Skill::Storm => 14,
            Skill::PetalDash => 8,
            Skill::Blossom => 14,
            Skill::Unpredictable => 8,
            Skill::FoxSwipe => 18,
            Skill::Vanish => 8,
            Skill::Overdrive => 30,
            Skill::RocketDash => 8,
            Skill::Starbreaker => 75,
            _ => 0,
        }
    }

    pub fn charged(self) -> bool {
        self == Skill::WindStep
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Mage,
    Melee,
    Tank,
    Support,
}

impl Role {
    pub const ALL: [Role; 4] = [Role::Mage, Role::Melee, Role::Tank, Role::Support];

    pub fn name(self) -> &'static str {
        match self {
            Role::Mage => "Mage",
            Role::Melee => "Melee",
            Role::Tank => "Tank",
            Role::Support => "Support",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Passive {
    Grace,
    Hexed,
    Intent,
    Bond,
    Fox,
    Overcharge,
}

impl Passive {
    pub fn name(self) -> &'static str {
        match self {
            Passive::Grace => "Elven Grace",
            Passive::Hexed => "Hexed",
            Passive::Intent => "Sword Intent",
            Passive::Bond => "Spirit Bond",
            Passive::Fox => "Fox Form",
            Passive::Overcharge => "Overcharge",
        }
    }
}

pub struct Kit {
    pub passive: Passive,
    pub name: &'static str,
    pub role: Role,
    pub skin: &'static str,
    pub skills: [Skill; 4],
    pub hp: i32,
    pub hint: &'static str,
}

const MAGE: [Skill; 4] = [Skill::Needle, Skill::Ray, Skill::Ward, Skill::Judgement];
const MAGE_HINT: &str = "LMB needle   Q piercing ray   E ward shell   R judgement of the heavens   PASSIVE grace: needle/ray hit or ward block = speed burst";
const WITCH: [Skill; 4] = [Skill::Hex, Skill::Potion, Skill::Broom, Skill::Cauldron];
const WITCH_HINT: &str = "LMB hex bolt   Q potion toss   E broom dash   R cauldron burst   PASSIVE hexed: 3rd hit detonates + root";

pub const KITS: [Kit; 6] = [
    Kit { name: "Aspen", role: Role::Mage, passive: Passive::Grace, skin: "elf", skills: MAGE, hp: 150, hint: MAGE_HINT },
    Kit { name: "Rowan", role: Role::Mage, passive: Passive::Hexed, skin: "witch", skills: WITCH, hp: 150, hint: WITCH_HINT },
    Kit {
        name: "Yun",
        role: Role::Melee,
        passive: Passive::Intent,
        skin: "sword",
        skills: [Skill::Cut, Skill::Swallow, Skill::WindStep, Skill::Formation],
        hp: 120,
        hint: "LMB swift cut (x3, 3rd: sword intent wave)   Q swallow strike (hit: +1 E charge)   E wind step (2 charges)   R sword formation   PASSIVE intent: 4 hits = free wave + E charge",
    },
    Kit {
        name: "Hikaru",
        role: Role::Mage,
        passive: Passive::Bond,
        skin: "parasol",
        skills: [Skill::Talisman, Skill::Parasol, Skill::PetalDash, Skill::Blossom],
        hp: 142,
        hint: "LMB paper talisman   Q parasol flight (again: move it, walk over: pick up)   E petal dash / parasol out: spirit step to parasol   R blossom spin / parasol out: sakura tempest   PASSIVE bond: near parasol = speed + shield, pickup resets E",
    },
    Kit {
        name: "Ren",
        role: Role::Mage,
        passive: Passive::Fox,
        skin: "fox",
        skills: [Skill::FoxOrb, Skill::ShiningLight, Skill::BlindLight, Skill::Unpredictable],
        hp: 140,
        hint: "LMB spirit orb / fox: claw   Q shining light / fox: triple swipe   E blinding light (stun blast) / fox: fox-fire wisps (3 hits: slow)   R charge into fox form / fox: charge back into human form (each R has its own cooldown)   PASSIVE fox form: +30% speed, -20% damage taken, each form keeps its own cooldowns",
    },
    Kit {
        name: "Seren",
        role: Role::Mage,
        passive: Passive::Overcharge,
        skin: "seren",
        skills: [Skill::ManaShot, Skill::AzureLance, Skill::GravityWell, Skill::Overdrive],
        hp: 145,
        hint: "LMB mana shot / fighter: rune knuckle (x3, 3rd: blast)   Q azure lance (thin beam, through own well: wider +50%) / fighter: overcharged fists (4s: +60% punches, each punch fires a wave)   E gravity well (pulls, then pops) / fighter: rocket dash (next punch is the blast)   R overdrive burst into fighter / fighter: starbreaker beam back into mage   PASSIVE overcharge: hits build 3 charges, full charges empower R (+40%), fighter form +20% speed, -15% damage taken",
    },
];

pub const WIND_CHARGES: u32 = 2;

pub fn kit(i: usize) -> &'static Kit {
    &KITS[i.min(KITS.len() - 1)]
}
