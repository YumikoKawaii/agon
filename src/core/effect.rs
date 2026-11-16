#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Slow,
    Root,
    Bleed,
    Burn,
    Mark,
    Shield,
    Vulnerable,
    Stun,
}

pub const N: usize = 8;
pub const KINDS: [EffectKind; N] = [
    EffectKind::Slow,
    EffectKind::Root,
    EffectKind::Bleed,
    EffectKind::Burn,
    EffectKind::Mark,
    EffectKind::Shield,
    EffectKind::Vulnerable,
    EffectKind::Stun,
];

pub const BLEED_MAX: u8 = 3;
const DOT_EVERY: u32 = 30;

#[derive(Clone, Copy, Debug)]
pub struct Apply(pub EffectKind, pub u32, pub i32);

impl EffectKind {
    pub fn i(self) -> usize {
        self as usize
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Effects {
    pub ticks: [u32; N],
    pub max: [u32; N],
    pub power: [i32; N],
    pub stacks: u8,
}

pub struct Dot {
    pub kind: EffectKind,
    pub dmg: i32,
}

impl Effects {
    pub fn on(&self, k: EffectKind) -> bool {
        self.ticks[k.i()] > 0
    }

    pub fn left(&self, k: EffectKind) -> u32 {
        self.ticks[k.i()]
    }

    pub fn progress(&self, k: EffectKind) -> f32 {
        let i = k.i();
        1.0 - self.ticks[i] as f32 / self.max[i].max(1) as f32
    }

    pub fn add(&mut self, a: Apply) {
        let Apply(k, ticks, power) = a;
        let i = k.i();
        match k {
            EffectKind::Bleed => {
                self.stacks = (self.stacks + 1).min(BLEED_MAX);
                self.power[i] = self.power[i].max(power);
            }
            EffectKind::Shield => self.power[i] += power,
            _ => self.power[i] = self.power[i].max(power),
        }
        if ticks >= self.ticks[i] {
            self.ticks[i] = ticks;
            self.max[i] = ticks;
        }
    }

    pub fn clear(&mut self, k: EffectKind) {
        let i = k.i();
        self.ticks[i] = 0;
        self.power[i] = 0;
        if k == EffectKind::Bleed {
            self.stacks = 0;
        }
    }

    pub fn reset(&mut self) {
        *self = Effects::default();
    }

    pub fn speed_mul(&self) -> f32 {
        if self.on(EffectKind::Slow) {
            1.0 - self.power[EffectKind::Slow.i()] as f32 / 100.0
        } else {
            1.0
        }
    }

    pub fn rooted(&self) -> bool {
        self.on(EffectKind::Root) || self.stunned()
    }

    pub fn stunned(&self) -> bool {
        self.on(EffectKind::Stun)
    }

    pub fn incoming(&mut self, dmg: i32) -> (i32, i32, bool) {
        let mut dmg = dmg;
        let mut marked = false;
        if self.on(EffectKind::Mark) {
            dmg += self.power[EffectKind::Mark.i()];
            self.clear(EffectKind::Mark);
            marked = true;
        }
        if self.on(EffectKind::Vulnerable) {
            dmg = dmg * (100 + self.power[EffectKind::Vulnerable.i()]) / 100;
        }
        let absorbed = self.absorb(dmg);
        (dmg - absorbed, absorbed, marked)
    }

    fn absorb(&mut self, dmg: i32) -> i32 {
        let i = EffectKind::Shield.i();
        if self.ticks[i] == 0 {
            return 0;
        }
        let a = self.power[i].min(dmg);
        self.power[i] -= a;
        if self.power[i] <= 0 {
            self.clear(EffectKind::Shield);
        }
        a
    }

    pub fn tick(&mut self) -> Vec<Dot> {
        let mut dots = Vec::new();
        for k in KINDS {
            let i = k.i();
            if self.ticks[i] == 0 {
                continue;
            }
            self.ticks[i] -= 1;
            let due = self.ticks[i] % DOT_EVERY == 0;
            match k {
                EffectKind::Bleed if due => dots.push(Dot { kind: k, dmg: self.power[i] * self.stacks as i32 }),
                EffectKind::Burn if due => dots.push(Dot { kind: k, dmg: self.power[i] }),
                _ => {}
            }
            if self.ticks[i] == 0 {
                self.clear(k);
            }
        }
        dots
    }

    pub fn pop_bleed(&mut self) -> u8 {
        let s = self.stacks;
        self.clear(EffectKind::Bleed);
        s
    }

    pub fn sync(&mut self, ticks: [u16; N], power: [u8; N], stacks: u8) {
        for i in 0..N {
            let t = ticks[i] as u32;
            if t > self.ticks[i] {
                self.max[i] = t;
            }
            self.ticks[i] = t;
            self.power[i] = power[i] as i32;
        }
        self.stacks = stacks;
    }

    pub fn wire(&self) -> ([u16; N], [u8; N], u8) {
        let mut t = [0u16; N];
        let mut p = [0u8; N];
        for i in 0..N {
            t[i] = self.ticks[i].min(u16::MAX as u32) as u16;
            p[i] = self.power[i].clamp(0, 255) as u8;
        }
        (t, p, self.stacks)
    }
}
