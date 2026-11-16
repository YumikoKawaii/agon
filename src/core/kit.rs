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
}

impl Skill {
    pub fn cooldown(self) -> f32 {
        match self {
            Skill::Needle => 0.8,
            Skill::Ray => 2.5,
            Skill::Ward => 4.0,
            Skill::Judgement => 7.0,
            Skill::Cut => 0.6,
            Skill::Swallow => 2.5,
            Skill::WindStep => 2.0,
            Skill::Formation => 8.0,
            Skill::Hex => 0.8,
            Skill::Potion => 2.5,
            Skill::Broom => 3.5,
            Skill::Cauldron => 8.0,
            Skill::Slash => 0.45,
            Skill::Whirl => 3.0,
            Skill::Cable => crate::core::warden::CABLE_GAP,
            Skill::Fury => 14.0,
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

pub struct Kit {
    pub name: &'static str,
    pub role: Role,
    pub skin: &'static str,
    pub skills: [Skill; 4],
    pub hp: i32,
    pub hint: &'static str,
}

const MAGE: [Skill; 4] = [Skill::Needle, Skill::Ray, Skill::Ward, Skill::Judgement];
const MAGE_HINT: &str = "LMB needle   Q piercing ray   E ward shell   R judgement circle";
const WITCH: [Skill; 4] = [Skill::Hex, Skill::Potion, Skill::Broom, Skill::Cauldron];
const WITCH_HINT: &str = "LMB hex bolt   Q potion toss   E broom dash   R cauldron burst";

pub const KITS: [Kit; 3] = [
    Kit { name: "Elf Mage", role: Role::Mage, skin: "elf", skills: MAGE, hp: 100, hint: MAGE_HINT },
    Kit { name: "Witch", role: Role::Mage, skin: "witch", skills: WITCH, hp: 100, hint: WITCH_HINT },
    Kit {
        name: "Sword Cultivator",
        role: Role::Melee,
        skin: "sword",
        skills: [Skill::Cut, Skill::Swallow, Skill::WindStep, Skill::Formation],
        hp: 80,
        hint: "LMB swift cut (x3, 3rd: sword intent wave)   Q swallow strike (hit: +1 E charge)   E wind step (2 charges)   R sword formation",
    },
];

pub const WIND_CHARGES: u32 = 2;

pub fn kit(i: usize) -> &'static Kit {
    &KITS[i.min(KITS.len() - 1)]
}
