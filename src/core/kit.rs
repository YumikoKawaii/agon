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
        }
    }

    pub fn lock_ticks(self) -> u32 {
        match self {
            Skill::Ray => 30,
            Skill::Judgement => 24,
            Skill::Formation => 26,
            Skill::Swallow => 12,
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

pub const KITS: [Kit; 3] = [
    Kit { name: "Elf Mage", skin: "elf", skills: MAGE, hp: 100, hint: MAGE_HINT },
    Kit { name: "Witch", skin: "witch", skills: MAGE, hp: 100, hint: MAGE_HINT },
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
