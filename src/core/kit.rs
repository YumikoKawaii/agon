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
    Slash,
    Whirl,
    Cable,
    Fury,
    Talisman,
    Parasol,
    Storm,
    SpiritStep,
}

impl Skill {
    pub fn cooldown(self) -> f32 {
        match self {
            Skill::Needle => 1.2,
            Skill::Ray => 2.5,
            Skill::Ward => 4.0,
            Skill::Judgement => 9.0,
            Skill::Cut => 0.6,
            Skill::Swallow => 2.5,
            Skill::WindStep => 2.0,
            Skill::Formation => 8.0,
            Skill::Hex => 1.2,
            Skill::Potion => 3.0,
            Skill::Broom => 3.5,
            Skill::Cauldron => 10.0,
            Skill::Slash => 0.45,
            Skill::Whirl => 3.0,
            Skill::Cable => crate::core::warden::CABLE_GAP,
            Skill::Fury => 14.0,
            Skill::Talisman => 1.1,
            Skill::Parasol => 1.0,
            Skill::Storm => 13.0,
            Skill::SpiritStep => 3.0,
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
            Skill::Slash => 12,
            Skill::Whirl => 18,
            Skill::Cable => 8,
            Skill::Fury => 20,
            Skill::Talisman => 16,
            Skill::Parasol => 14,
            Skill::Storm => 24,
            Skill::SpiritStep => 16,
        }
    }

    pub fn lock_ticks(self) -> u32 {
        match self {
            Skill::Ray => 30,
            Skill::Judgement => 24,
            Skill::Formation => 26,
            Skill::Swallow => 12,
            Skill::Cauldron => 20,
            Skill::Whirl => 16,
            Skill::Fury => 10,
            Skill::SpiritStep => 8,
            Skill::Storm => 14,
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
}

impl Passive {
    pub fn name(self) -> &'static str {
        match self {
            Passive::Grace => "Elven Grace",
            Passive::Hexed => "Hexed",
            Passive::Intent => "Sword Intent",
            Passive::Bond => "Spirit Bond",
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

pub const KITS: [Kit; 4] = [
    Kit { name: "Elf Mage", role: Role::Mage, passive: Passive::Grace, skin: "elf", skills: MAGE, hp: 150, hint: MAGE_HINT },
    Kit { name: "Witch", role: Role::Mage, passive: Passive::Hexed, skin: "witch", skills: WITCH, hp: 150, hint: WITCH_HINT },
    Kit {
        name: "Sword Cultivator",
        role: Role::Melee,
        passive: Passive::Intent,
        skin: "sword",
        skills: [Skill::Cut, Skill::Swallow, Skill::WindStep, Skill::Formation],
        hp: 120,
        hint: "LMB swift cut (x3, 3rd: sword intent wave)   Q swallow strike (hit: +1 E charge)   E wind step (2 charges)   R sword formation   PASSIVE intent: 4 hits = free wave + E charge",
    },
    Kit {
        name: "Parasol Onmyoji",
        role: Role::Mage,
        passive: Passive::Bond,
        skin: "parasol",
        skills: [Skill::Talisman, Skill::Parasol, Skill::SpiritStep, Skill::Storm],
        hp: 142,
        hint: "LMB paper talisman   Q parasol flight (again: move it, walk over: pick up)   E spirit step (to parasol)   R sakura tempest   PASSIVE bond: near parasol = speed + shield, pickup resets E",
    },
];

pub const WIND_CHARGES: u32 = 2;

pub fn kit(i: usize) -> &'static Kit {
    &KITS[i.min(KITS.len() - 1)]
}
