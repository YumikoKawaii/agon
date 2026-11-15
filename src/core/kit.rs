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
        }
    }

    pub fn lock_ticks(self) -> u32 {
        match self {
            Skill::Ray => 30,
            Skill::Judgement => 24,
            Skill::Formation => 26,
            Skill::Swallow => 12,
            Skill::Cauldron => 20,
            _ => 0,
        }
    }

    pub fn charged(self) -> bool {
        self == Skill::WindStep
    }
}

pub struct Kit {
    pub name: &'static str,
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
    Kit { name: "Elf Mage", skin: "elf", skills: MAGE, hp: 100, hint: MAGE_HINT },
    Kit { name: "Witch", skin: "witch", skills: WITCH, hp: 100, hint: WITCH_HINT },
    Kit {
        name: "Sword Cultivator",
        skin: "sword",
        skills: [Skill::Cut, Skill::Swallow, Skill::WindStep, Skill::Formation],
        hp: 80,
        hint: "LMB swift cut (x3)   Q swallow strike (hit: +1 E charge)   E wind step (2 charges)   R sword formation",
    },
];

pub const WIND_CHARGES: u32 = 2;

pub fn kit(i: usize) -> &'static Kit {
    &KITS[i.min(KITS.len() - 1)]
}
