use std::f32::consts::TAU;

use macroquad::{prelude::*, rand::gen_range};

use crate::{
    assets,
    core::{
        anim::State,
        effect::EffectKind,
        event::{Element, Event},
        magic::{ward_segment, JUDGE_R, NEEDLE_R},
        hex::{CAULDRON_R, POTION_R},
        player::dir8,
        sword::{FORM_N, FORM_R, INTENT_R, LUNGE_REACH},
        Battle, Player,
    },
    screen::battle::view::{alpha, draw_sprite, glow, status_color, Camera, AQUA, BLOOD, CHEST, CRIMSON, GOLD, ICE, LILAC, MINT, PALE, S, STEEL, VIEW},
    core::warden::{WHIRL_R, SLASH_R, SLASH_HALF},
    core::parasol::STORM_R,
    screen::battle::view::SAKURA,
};

const LEAF: Color = Color::new(0.55, 1.0, 0.82, 1.0);
const MOON: Color = Color::new(0.8, 0.68, 1.0, 1.0);

const FOX_MINT: Color = Color::new(0.498, 0.89, 0.784, 1.0);
const FOX_LAV: Color = Color::new(0.718, 0.608, 0.878, 1.0);
const SPIN_LIFE: f32 = 0.3;
const ZIP_LIFE: f32 = 0.3;
const FURY_GHOST: Color = Color::new(1.0, 0.35, 0.42, 1.0);

struct Spin {
    owner: usize,
    t: f32,
    big: bool,
    rot: f32,
}

const SHAKE_DECAY: f32 = 0.86;
const NUM_LIFE: f32 = 0.8;
const PILLAR_LIFE: f32 = 0.5;
const ARC_LIFE: f32 = 0.16;
const SLASH_LIFE: f32 = 0.25;
const GHOST_TINT: Color = Color::new(0.45, 1.0, 0.92, 1.0);

struct Part {
    pos: Vec2,
    z: f32,
    vel: Vec2,
    vz: f32,
    life: f32,
    max: f32,
    size: f32,
    col: Color,
    drag: f32,
}

struct Ring {
    pos: Vec2,
    z: f32,
    r0: f32,
    r1: f32,
    t: f32,
    max: f32,
    col: Color,
    w: f32,
}

struct Arc {
    owner: usize,
    ang: f32,
    r: f32,
    half: f32,
    side: f32,
    t: f32,
    big: bool,
    col: Color,
}

struct Slash {
    a: Vec2,
    b: Vec2,
    t: f32,
    col: Color,
    max: f32,
}

struct Ghost {
    seat: usize,
    pos: Vec2,
    ang: f32,
    t: f32,
}

struct Num {
    pos: Vec2,
    t: f32,
    dmg: i32,
    tag: Option<Color>,
}

struct Pillar {
    pos: Vec2,
    t: f32,
}

fn element_color(e: Element) -> Color {
    match e {
        Element::Frost => LEAF,
        Element::Holy => GOLD,
        Element::Blade => AQUA,
        Element::Hex => LILAC,
        Element::Spirit => SAKURA,
        Element::Arcane => ICE,
    }
}

fn petal() -> Color {
    match gen_range(0, 5) {
        0 => WHITE,
        1 => LILAC,
        _ => SAKURA,
    }
}

fn pick(a: Color, b: Color) -> Color {
    if gen_range(0, 2) == 0 { a } else { b }
}

#[derive(Clone, Copy)]
enum Kind {
    Potion,
    Burst,
    Poof,
    BleedPop,
    Release,
    Bloom,
    Step,
    Tempest,
    Dash,
    Blossom,
    Return,
    FoxBlast,
    FoxStrike,
    FoxSwipe,
    FoxPoof,
    FoxDash,
    SerenBlast,
    SerenBigBlast,
    SerenThrust,
    SerenLanceHit,
    SerenPop,
    SerenSigil,
}

struct Sprite {
    kind: Kind,
    pos: Vec2,
    t: f32,
    ang: f32,
}

fn sheet(kind: Kind) -> (&'static assets::Sheet, f32) {
    let w = &assets::get().witch;
    match kind {
        Kind::Potion => (&w.potion, 0.7),
        Kind::Burst => (&w.burst, 0.85),
        Kind::Poof => (&w.poof, 0.6),
        Kind::BleedPop => (&assets::get().bleed_pop.sheet, 0.6),
        Kind::Release => (&assets::get().sword.release, 0.6),
        Kind::Bloom => (&assets::get().parasol.bloom, 0.5),
        Kind::Step => (&assets::get().parasol.step, 0.6),
        Kind::Tempest => (&assets::get().parasol.tempest, STORM_R / 52.0),
        Kind::Dash => (&assets::get().parasol.dash, 1.0),
        Kind::Blossom => (&assets::get().parasol.blossom, crate::core::parasol::BLOSSOM_R / 60.0),
        Kind::Return => (&assets::get().parasol.ret, 0.8),
        Kind::FoxBlast => (&assets::get().fox.blast, crate::core::fox::BLIND_BLAST / 60.0),
        Kind::FoxStrike => (&assets::get().fox.strike, crate::core::fox::SHINE_R / 60.0),
        Kind::FoxSwipe => (&assets::get().fox.swipe, crate::core::fox::SWIPE_R / 35.0),
        Kind::FoxPoof => (&assets::get().fox.poof, 0.6),
        Kind::FoxDash => (&assets::get().fox.dash, 0.8),
        Kind::SerenBlast => (&assets::get().seren.blast, 0.55),
        Kind::SerenBigBlast => (&assets::get().seren.blast, 0.9),
        Kind::SerenThrust => (&assets::get().seren.thrust, 0.6),
        Kind::SerenLanceHit => (&assets::get().seren.lance, 0.7),
        Kind::SerenPop => (&assets::get().seren.pop, crate::core::seren::WELL_R / 48.0),
        Kind::SerenSigil => (&assets::get().seren.sigil, crate::core::seren::BURST_R / 58.0),
    }
}

#[derive(Default)]
pub struct Fx {
    sprites: Vec<Sprite>,
    parts: Vec<Part>,
    rings: Vec<Ring>,
    arcs: Vec<Arc>,
    slashes: Vec<Slash>,
    ghosts: Vec<Ghost>,
    nums: Vec<Num>,
    pillars: Vec<Pillar>,
    spins: Vec<Spin>,
    pub shake: f32,
    flash: f32,
}

impl Fx {
    #[allow(clippy::too_many_arguments)]
    fn part(&mut self, pos: Vec2, z: f32, vel: Vec2, vz: f32, life: f32, size: f32, col: Color, drag: f32) {
        self.parts.push(Part { pos, z, vel, vz, life, max: life, size, col, drag });
    }

    fn burst(&mut self, pos: Vec2, z: f32, n: usize, speed: f32, life: f32, size: f32, col: Color) {
        for _ in 0..n {
            let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(speed * 0.3, speed);
            self.part(pos, z, v, gen_range(-speed * 0.3, speed * 0.6), gen_range(life * 0.5, life), gen_range(size * 0.5, size), col, 0.9);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn ring(&mut self, pos: Vec2, z: f32, r0: f32, r1: f32, max: f32, col: Color, w: f32) {
        self.rings.push(Ring { pos, z, r0, r1, t: max, max, col, w });
    }

    fn petals(&mut self, pos: Vec2, z: f32, n: usize, speed: f32, life: f32) {
        for _ in 0..n {
            let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(speed * 0.3, speed);
            self.part(pos, z, v, gen_range(10.0, speed * 0.6), gen_range(life * 0.5, life), gen_range(1.0, 2.4), petal(), 0.9);
        }
    }

    fn bloom(&mut self, pos: Vec2) {
        self.sprites.push(Sprite { kind: Kind::Bloom, pos, t: 0.0, ang: 0.0 });
    }

    fn kick(&mut self, shake: f32, flash: f32) {
        self.shake = self.shake.max(shake);
        self.flash = self.flash.max(flash);
    }

    pub fn spawn(&mut self, event: &Event, players: &[Player; 2]) {
        match *event {
            Event::Needle { tip, ang } => {
                for _ in 0..6 {
                    let v = Vec2::from_angle(ang + gen_range(-0.6, 0.6)) * gen_range(40.0, 110.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 20.0), 0.25, 1.5, MOON, 0.85);
                }
                self.ring(tip, CHEST, 1.0, 7.0, 0.15, LEAF, 2.0);
            }
            Event::Ward { pos, front } => {
                self.ring(front, CHEST, 2.0, 40.0, 0.4, ICE, 2.0);
                self.ring(pos, CHEST, 4.0, 30.0, 0.35, PALE, 3.0);
                self.burst(front, CHEST, 16, 60.0, 0.4, 2.0, ICE);
            }
            Event::Blocked { pos } => {
                self.burst(pos, CHEST, 18, 120.0, 0.35, 2.0, PALE);
                self.ring(pos, CHEST, 2.0, 16.0, 0.25, ICE, 3.0);
                self.kick(2.5, 0.0);
            }
            Event::RayFired { origin, dir } => {
                self.ring(origin + dir * 8.0, CHEST, 6.0, 46.0, 0.35, ICE, 3.0);
                self.ring(origin + dir * crate::core::magic::RAY_LEN, CHEST, 3.0, 30.0, 0.3, PALE, 3.0);
                for _ in 0..50 {
                    let along = origin + dir * gen_range(10.0, crate::core::magic::RAY_LEN);
                    let v = dir.perp() * gen_range(-90.0, 90.0) + dir * gen_range(20.0, 120.0);
                    let col = if gen_range(0, 3) == 0 { WHITE } else { ICE };
                    self.part(along, CHEST, v, gen_range(-20.0, 40.0), gen_range(0.2, 0.5), gen_range(1.0, 2.5), col, 0.88);
                }
                self.ring(origin + dir * 8.0, CHEST, 3.0, 22.0, 0.25, PALE, 4.0);
                self.kick(8.0, 0.2);
            }
            Event::JudgeCast { caster, at } => {
                self.ring(at, 0.0, JUDGE_R * 2.0, JUDGE_R, 0.5, GOLD, 3.0);
                self.ring(caster, 0.0, 4.0, 40.0, 0.5, GOLD, 3.0);
                for _ in 0..40 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(10.0, 50.0);
                    self.part(caster, gen_range(0.0, 10.0), v, gen_range(80.0, 180.0), gen_range(0.4, 0.8), gen_range(1.0, 2.5), pick(GOLD, WHITE), 0.92);
                }
                self.kick(2.0, 0.0);
            }
            Event::JudgeStrike { pos } => {
                self.pillars.push(Pillar { pos, t: PILLAR_LIFE });
                self.ring(pos, 0.0, 4.0, JUDGE_R * 2.4, 0.6, GOLD, 7.0);
                self.ring(pos, 0.0, 4.0, JUDGE_R * 1.7, 0.45, WHITE, 5.0);
                self.ring(pos, 0.0, 4.0, JUDGE_R * 1.2, 0.35, ICE, 4.0);
                self.ring(pos, CHEST, 4.0, JUDGE_R, 0.3, PALE, 3.0);
                self.ring(pos, 40.0, 2.0, JUDGE_R * 0.8, 0.3, GOLD, 3.0);
                for _ in 0..160 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(60.0, 260.0);
                    let col = match gen_range(0, 4) { 0 => WHITE, 1 => ICE, _ => GOLD };
                    self.part(pos, gen_range(0.0, 12.0), v, gen_range(20.0, 260.0), gen_range(0.4, 1.0), gen_range(1.5, 3.8), col, 0.92);
                }
                for _ in 0..30 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(5.0, 30.0);
                    self.part(pos, 0.0, v, gen_range(200.0, 360.0), gen_range(0.5, 0.9), gen_range(1.5, 2.5), WHITE, 0.95);
                }
                self.kick(18.0, 0.5);
            }
            Event::Cut { owner, ang, r, half, side, big } => {
                self.arcs.push(Arc { owner, ang, r, half, side, t: ARC_LIFE, big, col: AQUA });
                let pos = players[owner].pos;
                for _ in 0..(if big { 14 } else { 6 }) {
                    let a = ang + gen_range(-half, half);
                    self.part(pos + Vec2::from_angle(a) * r * 0.8, CHEST, Vec2::from_angle(a) * gen_range(30.0, 90.0), gen_range(0.0, 20.0), 0.25, 1.5, pick(AQUA, WHITE), 0.88);
                }
                if big {
                    self.kick(3.0, 0.0);
                }
            }
            Event::Swallow { pos, dir } => {
                self.ring(pos, CHEST, 2.0, 14.0, 0.2, AQUA, 3.0);
                for _ in 0..10 {
                    let v = -dir * gen_range(40.0, 120.0) + dir.perp() * gen_range(-50.0, 50.0);
                    self.part(pos, CHEST, v, gen_range(0.0, 20.0), 0.3, 1.6, PALE, 0.88);
                }
            }
            Event::Intent { pos, dir } => {
                self.sprites.push(Sprite { kind: Kind::Release, pos: pos + dir * 8.0, t: 0.0, ang: 0.0 });
                self.ring(pos, CHEST, 4.0, 26.0, 0.3, AQUA, 4.0);
                for _ in 0..24 {
                    let v = dir * gen_range(60.0, 200.0) + dir.perp() * gen_range(-80.0, 80.0);
                    self.part(pos, CHEST, v, gen_range(0.0, 30.0), 0.35, 1.8, pick(AQUA, LILAC), 0.88);
                }
                self.kick(5.0, 0.06);
            }
            Event::IntentFade { pos, dir } => {
                for _ in 0..16 {
                    let a = gen_range(-1.0, 1.0);
                    let p = pos + dir.perp() * a * INTENT_R;
                    self.part(p, CHEST, dir * gen_range(20.0, 70.0), gen_range(0.0, 20.0), 0.35, 1.6, pick(AQUA, PALE), 0.9);
                }
            }
            Event::SwallowHit { tip } => {
                self.ring(tip, CHEST, 2.0, 20.0, 0.3, WHITE, 4.0);
                self.burst(tip, CHEST, 16, 140.0, 0.35, 2.0, AQUA);
            }
            Event::Wind { from, to, dir } => {
                self.slashes.push(Slash { a: from, b: to, t: SLASH_LIFE, col: AQUA, max: SLASH_LIFE });
                for _ in 0..16 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-6.0, 6.0), dir.perp() * gen_range(-60.0, 60.0) - dir * 30.0, gen_range(0.0, 30.0), 0.35, 1.6, pick(AQUA, PALE), 0.88);
                }
            }
            Event::Ghost { seat, pos, ang, life } => self.ghosts.push(Ghost { seat, pos, ang, t: life }),
            Event::FormCast { caster, center } => {
                self.ring(center, 0.0, FORM_R * 2.0, FORM_R, 0.4, AQUA, 3.0);
                self.burst(caster, CHEST + 6.0, 24, 90.0, 0.4, 2.0, AQUA);
            }
            Event::FormBurst { center, base } => {
                self.ring(center, 0.0, 4.0, FORM_R * 2.2, 0.45, AQUA, 6.0);
                self.ring(center, CHEST, 4.0, FORM_R * 1.4, 0.3, WHITE, 4.0);
                for n in 0..FORM_N {
                    let d = Vec2::from_angle(base + n as f32 * TAU / FORM_N as f32);
                    self.burst(center - d * FORM_R, CHEST, 6, 90.0, 0.4, 2.0, PALE);
                }
                self.burst(center, CHEST, 50, 180.0, 0.6, 2.6, AQUA);
                self.kick(9.0, 0.18);
            }
            Event::HexCast { tip } => {
                self.burst(tip, CHEST, 8, 50.0, 0.3, 1.8, LILAC);
            }
            Event::PotionBurst { pos } => {
                self.sprites.push(Sprite { kind: Kind::Potion, pos, t: 0.0, ang: 0.0 });
                self.ring(pos, 0.0, 4.0, POTION_R * 1.6, 0.4, LILAC, 5.0);
                self.burst(pos, 6.0, 40, 150.0, 0.5, 2.4, LILAC);
                self.burst(pos, 6.0, 14, 90.0, 0.4, 1.6, WHITE);
                self.kick(5.0, 0.0);
            }
            Event::Broom { from, to, dir } => {
                self.sprites.push(Sprite { kind: Kind::Poof, pos: from, t: 0.0, ang: 0.0 });
                self.sprites.push(Sprite { kind: Kind::Poof, pos: to, t: 0.0, ang: 0.0 });
                for _ in 0..18 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-6.0, 6.0), dir.perp() * gen_range(-50.0, 50.0) - dir * 30.0, gen_range(0.0, 30.0), 0.4, 1.8, pick(LILAC, GOLD), 0.88);
                }
            }
            Event::CauldronCast { caster, at } => {
                self.ring(at, 0.0, CAULDRON_R * 1.6, CAULDRON_R, 0.4, LILAC, 3.0);
                self.burst(caster, CHEST + 8.0, 20, 70.0, 0.5, 2.0, LILAC);
            }
            Event::CauldronBurst { pos } => {
                self.sprites.push(Sprite { kind: Kind::Burst, pos, t: 0.0, ang: 0.0 });
                self.ring(pos, 0.0, 4.0, CAULDRON_R * 1.8, 0.5, LILAC, 6.0);
                self.ring(pos, 0.0, 4.0, CAULDRON_R * 1.2, 0.35, WHITE, 4.0);
                for _ in 0..80 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(50.0, 200.0);
                    self.part(pos, gen_range(0.0, 8.0), v, gen_range(40.0, 220.0), gen_range(0.4, 0.9), gen_range(1.5, 3.5), pick(LILAC, AQUA), 0.92);
                }
                self.kick(12.0, 0.25);
            }
            Event::Hit { pos, element, dmg, big } => {
                if element == Element::Spirit {
                    self.bloom(pos);
                }
                let col = element_color(element);
                let n = if big { 30 } else { 10 };
                self.burst(pos, CHEST, n, if big { 140.0 } else { 80.0 }, 0.45, if big { 3.0 } else { 2.0 }, col);
                self.burst(pos, CHEST, n / 2, 60.0, 0.3, 1.5, WHITE);
                self.ring(pos, CHEST, 2.0, if big { 26.0 } else { 12.0 }, 0.3, col, 3.0);
                self.nums.push(Num { pos: pos + vec2(gen_range(-4.0, 4.0), 0.0), t: NUM_LIFE, dmg, tag: None });
                self.kick(if big { 7.0 } else { 3.0 }, 0.0);
            }
            Event::Status { pos, kind } => {
                let col = status_color(kind);
                self.ring(pos, CHEST, 3.0, 20.0, 0.35, col, 3.0);
                self.burst(pos, CHEST, 14, 90.0, 0.5, 2.2, col);
                for n in 0..10 {
                    let a = n as f32 / 10.0 * TAU;
                    self.part(pos + Vec2::from_angle(a) * 12.0, 0.0, Vec2::from_angle(a) * -20.0, gen_range(60.0, 120.0), 0.5, 1.6, pick(col, WHITE), 0.92);
                }
                if kind == EffectKind::Stun || kind == EffectKind::Root {
                    self.kick(6.0, 0.08);
                }
            }
            Event::Dot { pos, kind, dmg } => {
                let col = status_color(kind);
                self.burst(pos, CHEST, 6, 40.0, 0.35, 1.6, col);
                self.nums.push(Num { pos: pos + vec2(gen_range(-6.0, 6.0), 0.0), t: NUM_LIFE, dmg, tag: Some(col) });
            }
            Event::Absorb { pos, amount } => {
                self.ring(pos, CHEST, 6.0, 22.0, 0.3, GOLD, 4.0);
                self.burst(pos, CHEST, 18, 120.0, 0.4, 2.0, pick(GOLD, PALE));
                self.nums.push(Num { pos: pos + vec2(0.0, -4.0), t: NUM_LIFE, dmg: -amount, tag: Some(GOLD) });
            }
            Event::MarkPop { pos } => {
                self.ring(pos, CHEST, 2.0, 34.0, 0.4, GOLD, 5.0);
                self.ring(pos, 0.0, 4.0, 30.0, 0.5, PALE, 3.0);
                self.burst(pos, CHEST, 30, 180.0, 0.6, 2.6, pick(GOLD, WHITE));
                self.kick(10.0, 0.2);
            }
            Event::BleedPop { pos, dmg } => {
                self.sprites.push(Sprite { kind: Kind::BleedPop, pos, t: 0.0, ang: 0.0 });
                self.burst(pos, CHEST, 20 + dmg as usize * 2, 160.0, 0.6, 2.4, pick(BLOOD, AQUA));
                self.ring(pos, CHEST, 2.0, 28.0, 0.3, BLOOD, 4.0);
                self.kick(9.0, 0.12);
            }
            Event::Twin { owner, ang, side, big } => {
                let r = if big { SLASH_R * 1.15 } else { SLASH_R };
                self.arcs.push(Arc { owner, ang, r, half: SLASH_HALF, side, t: ARC_LIFE, big, col: CRIMSON });
                self.arcs.push(Arc { owner, ang, r: r * 0.8, half: SLASH_HALF * 0.9, side: -side, t: ARC_LIFE * 0.85, big: false, col: STEEL });
                let pos = players[owner].pos;
                for _ in 0..(if big { 22 } else { 10 }) {
                    let a = ang + gen_range(-SLASH_HALF, SLASH_HALF);
                    self.part(pos + Vec2::from_angle(a) * r * 0.85, CHEST, Vec2::from_angle(a) * gen_range(40.0, 120.0), gen_range(0.0, 30.0), 0.3, 1.6, pick(CRIMSON, WHITE), 0.88);
                }
                if big {
                    self.ring(pos + Vec2::from_angle(ang) * r * 0.6, CHEST, 2.0, 18.0, 0.2, CRIMSON, 4.0);
                    self.kick(4.0, 0.0);
                }
            }
            Event::Whirl { owner, big } => {
                self.spins.push(Spin { owner, t: SPIN_LIFE, big, rot: gen_range(0.0, TAU) });
                let pos = players[owner].pos;
                self.ring(pos, 0.0, 6.0, WHIRL_R * 1.4, 0.35, CRIMSON, 4.0);
                self.ring(pos, CHEST, 4.0, WHIRL_R, 0.25, STEEL, 3.0);
                if big {
                    self.ring(pos, CHEST, 2.0, WHIRL_R * 1.6, 0.3, WHITE, 5.0);
                    self.kick(7.0, 0.1);
                }
            }
            Event::WhirlPulse { owner, big } => {
                let pos = players[owner].pos;
                for n in 0..(if big { 28 } else { 16 }) {
                    let a = n as f32 / if big { 28.0 } else { 16.0 } * TAU + gen_range(-0.1, 0.1);
                    let d = Vec2::from_angle(a);
                    self.part(pos + d * WHIRL_R * 0.9, CHEST + gen_range(-3.0, 3.0), d.perp() * 140.0 + d * 40.0, gen_range(0.0, 30.0), 0.28, 1.7, pick(CRIMSON, STEEL), 0.86);
                }
                self.kick(if big { 4.0 } else { 2.0 }, 0.0);
            }
            Event::HookFire { from, to } => {
                let d = (to - from).normalize_or(Vec2::X);
                self.ring(from + d * 6.0, CHEST, 2.0, 12.0, 0.18, STEEL, 3.0);
                for _ in 0..12 {
                    let v = -d * gen_range(30.0, 90.0) + d.perp() * gen_range(-50.0, 50.0);
                    self.part(from, CHEST - 3.0, v, gen_range(10.0, 50.0), gen_range(0.3, 0.6), gen_range(1.5, 3.0), alpha(PALE, 0.7), 0.9);
                }
            }
            Event::HookBite { pos } => {
                self.ring(pos, CHEST, 2.0, 16.0, 0.22, WHITE, 4.0);
                self.burst(pos, CHEST, 14, 120.0, 0.3, 1.6, pick(STEEL, GOLD));
                self.kick(2.5, 0.0);
            }
            Event::Zip { from, to } => {
                self.slashes.push(Slash { a: from, b: to, t: ZIP_LIFE, col: CRIMSON, max: ZIP_LIFE });
                let dir = (to - from).normalize_or(Vec2::X);
                for _ in 0..22 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-6.0, 6.0), dir.perp() * gen_range(-60.0, 60.0) - dir * 40.0, gen_range(0.0, 30.0), 0.4, 1.8, pick(CRIMSON, PALE), 0.88);
                }
                self.ring(to, 0.0, 4.0, 18.0, 0.25, CRIMSON, 3.0);
            }
            Event::Fury { pos } => {
                self.pillars.push(Pillar { pos, t: PILLAR_LIFE * 0.7 });
                self.ring(pos, 0.0, 4.0, 60.0, 0.6, CRIMSON, 7.0);
                self.ring(pos, 0.0, 4.0, 40.0, 0.45, WHITE, 4.0);
                self.ring(pos, CHEST, 30.0, 4.0, 0.35, CRIMSON, 4.0);
                for _ in 0..90 {
                    let v = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(40.0, 200.0);
                    self.part(pos, gen_range(0.0, 10.0), v, gen_range(40.0, 220.0), gen_range(0.4, 0.9), gen_range(1.5, 3.2), pick(CRIMSON, BLOOD), 0.92);
                }
                self.kick(12.0, 0.3);
            }
            Event::TalismanCast { tip, dir } => {
                self.bloom(tip + dir * 4.0);
                for _ in 0..8 {
                    let v = Vec2::from_angle(dir.y.atan2(dir.x) + gen_range(-0.7, 0.7)) * gen_range(40.0, 110.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 30.0), 0.3, gen_range(1.0, 1.8), petal(), 0.86);
                }
            }
            Event::ParasolThrow { from, to } => {
                self.bloom(from);
                self.ring(to, 0.0, 22.0, 6.0, 0.35, SAKURA, 2.0);
            }
            Event::ParasolPlant { pos } => {
                self.bloom(pos);
                self.ring(pos, 0.0, 2.0, 24.0, 0.35, SAKURA, 3.0);
                self.petals(pos, 8.0, 14, 90.0, 0.6);
                self.kick(3.0, 0.0);
            }
            Event::ParasolPickup { pos } => {
                self.bloom(pos);
                self.ring(pos, CHEST, 18.0, 2.0, 0.3, SAKURA, 3.0);
            }
            Event::SpiritStep { from, to, ang } => {
                self.sprites.push(Sprite { kind: Kind::Return, pos: from, t: 0.0, ang });
                self.sprites.push(Sprite { kind: Kind::Return, pos: to, t: 0.0, ang });
                self.kick(9.0, 0.12);
            }
            Event::PetalDash { from, to, ang } => {
                let _ = to;
                self.sprites.push(Sprite { kind: Kind::Dash, pos: from, t: 0.0, ang });
                self.kick(5.0, 0.05);
            }
            Event::Blossom { pos } => {
                self.sprites.push(Sprite { kind: Kind::Blossom, pos, t: 0.0, ang: 0.0 });
                self.kick(10.0, 0.2);
            }
            Event::StormCast { caster, at } => {
                self.bloom(caster);
                self.ring(at, 0.0, STORM_R * 2.2, STORM_R, 0.5, SAKURA, 3.0);
                self.kick(4.0, 0.05);
            }
            Event::StormTick => self.kick(1.5, 0.0),
            Event::StormBurst { pos } => {
                self.sprites.push(Sprite { kind: Kind::Tempest, pos, t: 0.0, ang: 0.0 });
                for n in 0..6 {
                    self.bloom(pos + Vec2::from_angle(n as f32 * TAU / 6.0) * STORM_R * 0.6);
                }
                self.petals(pos, 20.0, 60, 240.0, 1.0);
                self.kick(16.0, 0.4);
            }
            Event::FoxCast { tip, dir } => {
                for _ in 0..6 {
                    let v = Vec2::from_angle(dir.y.atan2(dir.x) + gen_range(-0.6, 0.6)) * gen_range(30.0, 90.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 25.0), 0.3, gen_range(1.0, 1.6), pick(FOX_MINT, FOX_LAV), 0.86);
                }
            }
            Event::FoxBlast { pos } => {
                self.sprites.push(Sprite { kind: Kind::FoxBlast, pos, t: 0.0, ang: 0.0 });
                self.kick(8.0, 0.1);
            }
            Event::FoxStrike { pos } => {
                self.sprites.push(Sprite { kind: Kind::FoxStrike, pos, t: 0.0, ang: 0.0 });
                self.kick(10.0, 0.15);
            }
            Event::FoxSwipe { pos, ang, big } => {
                self.sprites.push(Sprite { kind: Kind::FoxSwipe, pos, t: 0.0, ang });
                if big {
                    self.kick(4.0, 0.0);
                }
            }
            Event::FoxDash { from, to } => {
                let d = (to - from).normalize_or_zero();
                self.sprites.push(Sprite { kind: Kind::FoxDash, pos: from, t: 0.0, ang: d.y.atan2(d.x) });
                for _ in 0..14 {
                    let p = from.lerp(to, gen_range(0.0, 1.0));
                    self.part(p, CHEST + gen_range(-4.0, 4.0), d.perp() * gen_range(-30.0, 30.0) - d * 30.0, gen_range(0.0, 20.0), 0.4, 1.6, pick(FOX_MINT, FOX_LAV), 0.88);
                }
                self.kick(5.0, 0.05);
            }
            Event::FoxPoof { pos } => {
                self.sprites.push(Sprite { kind: Kind::FoxPoof, pos, t: 0.0, ang: 0.0 });
                self.kick(4.0, 0.05);
            }
            Event::SerenShot { tip, dir } => {
                for _ in 0..8 {
                    let v = Vec2::from_angle(dir.y.atan2(dir.x) + gen_range(-0.5, 0.5)) * gen_range(40.0, 110.0);
                    self.part(tip, CHEST, v, gen_range(-10.0, 25.0), 0.3, gen_range(1.0, 1.6), pick(ICE, GOLD), 0.86);
                }
                self.ring(tip, CHEST, 2.0, 9.0, 0.2, ICE, 1.5);
            }
            Event::SerenBlast { pos, big } => {
                let kind = if big { Kind::SerenBigBlast } else { Kind::SerenBlast };
                self.sprites.push(Sprite { kind, pos, t: 0.0, ang: 0.0 });
                self.burst(pos, CHEST, if big { 24 } else { 12 }, 110.0, 0.45, 1.6, ICE);
                self.kick(if big { 12.0 } else { 6.0 }, if big { 0.15 } else { 0.05 });
            }
            Event::SerenTransform { pos, over } => {
                self.sprites.push(Sprite { kind: Kind::SerenSigil, pos, t: 0.0, ang: 0.0 });
                for n in 0..16 {
                    let v = Vec2::from_angle(n as f32 * std::f32::consts::TAU / 16.0) * gen_range(150.0, 200.0);
                    self.part(pos + v * 0.06, CHEST, v, gen_range(10.0, 40.0), 0.35, gen_range(1.2, 1.8), pick(ICE, GOLD), 0.88);
                }
                self.kick(if over { 14.0 } else { 8.0 }, 0.2);
            }
            Event::SerenPunch { pos, ang, big, lit } => {
                let d = Vec2::from_angle(ang);
                if lit {
                    for _ in 0..10 {
                        let v = Vec2::from_angle(ang + gen_range(-0.4, 0.4)) * gen_range(90.0, 180.0);
                        self.part(pos, CHEST + gen_range(-3.0, 3.0), v, gen_range(10.0, 40.0), 0.35, gen_range(1.4, 2.2), pick(GOLD, WHITE), 0.85);
                    }
                    self.ring(pos + d * 6.0, CHEST, 4.0, if big { 26.0 } else { 15.0 }, 0.22, GOLD, 2.5);
                    self.kick(if big { 9.0 } else { 3.0 }, if big { 0.1 } else { 0.03 });
                }
                for _ in 0..if big { 14 } else { 6 } {
                    let v = Vec2::from_angle(ang + gen_range(-0.7, 0.7)) * gen_range(50.0, 130.0);
                    self.part(pos, CHEST, v, gen_range(0.0, 20.0), 0.25, gen_range(1.0, 1.8), pick(ICE, GOLD), 0.84);
                }
                self.ring(pos + d * 4.0, CHEST, 2.0, if big { 16.0 } else { 9.0 }, 0.18, ICE, 2.0);
                if big {
                    self.sprites.push(Sprite { kind: Kind::SerenBlast, pos: pos + d * 8.0, t: 0.0, ang: 0.0 });
                    self.kick(6.0, 0.05);
                }
            }
            Event::SerenLance { origin, dir, big, hit } => {
                let col = if big { GOLD } else { ICE };
                self.ring(origin, CHEST, 2.0, if big { 16.0 } else { 10.0 }, 0.2, col, 2.0);
                for _ in 0..if big { 30 } else { 16 } {
                    let p = origin + dir * gen_range(0.0, crate::core::seren::LANCE_RANGE);
                    self.part(p, CHEST, dir.perp() * gen_range(-25.0, 25.0) + dir * 30.0, gen_range(0.0, 15.0), 0.3, gen_range(0.8, 1.5), pick(col, WHITE), 0.85);
                }
                if let Some(p) = hit {
                    self.sprites.push(Sprite { kind: Kind::SerenLanceHit, pos: p, t: 0.0, ang: 0.0 });
                    self.burst(p, CHEST, 14, 120.0, 0.4, 1.4, ICE);
                }
                self.kick(if big { 8.0 } else { 3.0 }, if big { 0.08 } else { 0.0 });
            }
            Event::SerenWell { pos } => {
                self.ring(pos, 0.0, 50.0, 4.0, 0.35, MOON, 2.5);
                self.ring(pos, 0.0, 36.0, 2.0, 0.3, ICE, 1.5);
            }
            Event::SerenPop { pos } => {
                self.sprites.push(Sprite { kind: Kind::SerenPop, pos, t: 0.0, ang: 0.0 });
                self.ring(pos, 0.0, 4.0, crate::core::seren::WELL_R + 10.0, 0.4, MOON, 3.0);
                self.burst(pos, CHEST, 26, 150.0, 0.5, 1.8, pick(ICE, MOON));
                self.kick(10.0, 0.12);
            }
            Event::SerenFists { pos } => {
                self.ring(pos, CHEST, 26.0, 4.0, 0.3, GOLD, 2.5);
                self.ring(pos, CHEST, 4.0, 22.0, 0.35, ICE, 2.0);
                self.burst(pos, CHEST, 20, 90.0, 0.45, 1.6, pick(GOLD, ICE));
                self.kick(5.0, 0.08);
            }
            Event::SerenDash { from, to } => {
                let d = (to - from).normalize_or(Vec2::X);
                self.sprites.push(Sprite { kind: Kind::SerenThrust, pos: from, t: 0.0, ang: 0.0 });
                self.ring(from, 2.0, 4.0, 22.0, 0.3, ICE, 2.5);
                for n in 0..18 {
                    let p = from.lerp(to, n as f32 / 18.0);
                    self.part(p, CHEST + gen_range(-4.0, 4.0), -d * gen_range(20.0, 70.0) + d.perp() * gen_range(-20.0, 20.0), gen_range(0.0, 15.0), 0.4, gen_range(1.0, 1.8), pick(ICE, GOLD), 0.88);
                }
                for _ in 0..10 {
                    self.part(to, 2.0, Vec2::from_angle(gen_range(0.0, std::f32::consts::TAU)) * gen_range(40.0, 90.0), gen_range(10.0, 30.0), 0.3, 1.2, GOLD, 0.85);
                }
                self.kick(6.0, 0.05);
            }
            Event::SerenCharge { pos } => {
                self.ring(pos, CHEST, 18.0, 3.0, 0.8, ICE, 2.0);
                self.ring(pos, CHEST, 26.0, 4.0, 0.8, GOLD, 1.5);
            }
            Event::SerenBeam { over, .. } => {
                self.kick(if over { 22.0 } else { 14.0 }, if over { 0.35 } else { 0.2 });
            }
            Event::SerenFull { pos } => {
                self.ring(pos, CHEST, 2.0, 14.0, 0.4, GOLD, 2.0);
                self.burst(pos, CHEST, 10, 60.0, 0.4, 1.4, GOLD);
            }
            Event::Heal { pos, amount } => {
                self.burst(pos, CHEST, 10, 60.0, 0.4, 1.6, MINT);
                self.nums.push(Num { pos: pos + vec2(0.0, -6.0), t: NUM_LIFE, dmg: -amount, tag: Some(MINT) });
            }
        }
    }

    pub fn ambient(&mut self, b: &Battle) {
        for t in &b.parasols.talismans {
            self.part(t.pos, CHEST + gen_range(-1.5, 1.5), -t.vel * 0.15 + Vec2::from_angle(gen_range(0.0, TAU)) * 15.0, 0.0, 0.3, gen_range(0.8, 1.6), petal(), 0.9);
        }
        for fl in &b.parasols.flights {
            for _ in 0..3 {
                let off = Vec2::from_angle(fl.spin * 3.0 + gen_range(0.0, TAU)) * 7.0;
                self.part(fl.pos + off, CHEST + 6.0, off * 6.0, gen_range(0.0, 30.0), 0.4, gen_range(1.0, 2.0), petal(), 0.9);
            }
        }
        for at in b.parasols.planted.iter().flatten() {
            if gen_range(0, 4) == 0 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(4.0, 10.0);
                self.part(*at + off, 22.0, off * 0.5, -gen_range(5.0, 15.0), 0.9, gen_range(0.8, 1.4), petal(), 0.98);
            }
        }
        for s in &b.parasols.storms {
            let k = s.k();
            for _ in 0..(4 + (k * 8.0) as usize) {
                let a = gen_range(0.0, TAU);
                let r = STORM_R * gen_range(0.2, 1.05);
                let p = s.pos + Vec2::from_angle(a) * r;
                let swirl = Vec2::from_angle(a + 1.9) * gen_range(80.0, 160.0) * (0.5 + k);
                self.part(p, gen_range(0.0, 40.0 + 30.0 * k), swirl, gen_range(-20.0, 40.0), gen_range(0.35, 0.7), gen_range(1.0, 2.6), petal(), 0.9);
            }
        }
        for s in &b.spells.shots {
            let (pos, vel) = (s.pos, s.vel);
            self.part(pos, CHEST + gen_range(-1.0, 1.0), -vel * 0.1, 0.0, 0.25, NEEDLE_R * 0.7, ICE, 0.9);
        }
        for r in b.spells.rays.iter().filter(|r| !r.fired) {
            let tip = r.tip();
            for _ in 0..3 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(14.0, 26.0);
                self.part(tip + off, CHEST + gen_range(-8.0, 8.0), -off * 4.0, 0.0, 0.22, 1.8, pick(ICE, PALE), 1.0);
            }
        }
        for w in &b.spells.wards {
            if gen_range(0, 3) == 0 {
                let (a, c) = ward_segment(&b.players[w.owner]);
                self.part(a.lerp(c, gen_range(0.0, 1.0)), CHEST + gen_range(-6.0, 6.0), Vec2::ZERO, 18.0, 0.4, 1.3, PALE, 1.0);
            }
        }
        for j in &b.spells.judges {
            for _ in 0..(1 + (j.charge() * 4.0) as usize) {
                let p = j.pos + Vec2::from_angle(gen_range(0.0, TAU)) * JUDGE_R * gen_range(0.85, 1.0);
                self.part(p, 0.0, Vec2::ZERO, gen_range(40.0, 120.0), 0.5, gen_range(1.0, 2.2), pick(GOLD, PALE), 1.0);
            }
        }
        for w in &b.blades.waves {
            for _ in 0..3 {
                let p = w.pos + w.dir.perp() * gen_range(-1.0, 1.0) * INTENT_R * 0.9;
                self.part(p, CHEST + gen_range(-4.0, 4.0), -w.dir * gen_range(30.0, 90.0), 0.0, 0.3, 1.6, pick(AQUA, LILAC), 0.9);
            }
        }
        for p in b.players.iter().filter(|p| p.intent_ready()) {
            if gen_range(0, 2) == 0 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(6.0, 12.0);
                self.part(p.pos + off, gen_range(0.0, CHEST), -off * 1.5, gen_range(30.0, 70.0), 0.45, gen_range(1.0, 2.0), pick(AQUA, LILAC), 0.95);
            }
        }
        for h in &b.warden.hooks {
            if !h.pulling {
                self.part(h.pos, CHEST, Vec2::ZERO, 0.0, 0.12, 1.2, STEEL, 1.0);
            } else {
                let p = b.players[h.owner].pos;
                self.part(p, CHEST + gen_range(-4.0, 4.0), Vec2::from_angle(gen_range(0.0, TAU)) * 30.0, 10.0, 0.3, 1.8, pick(CRIMSON, PALE), 0.9);
            }
        }
        for p in b.players.iter().filter(|p| p.fury > 0.0) {
            for _ in 0..2 {
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(4.0, 12.0);
                self.part(p.pos + off, gen_range(0.0, 6.0), off * 0.5, gen_range(50.0, 110.0), gen_range(0.4, 0.7), gen_range(1.0, 2.2), pick(CRIMSON, GOLD), 0.95);
            }
        }
        for l in &b.blades.lunges {
            let tip = b.players[l.owner].pos + l.dir * LUNGE_REACH;
            self.part(tip, CHEST, -l.dir * 60.0 + l.dir.perp() * gen_range(-30.0, 30.0), 0.0, 0.25, 1.5, pick(AQUA, WHITE), 0.88);
        }
        for fm in &b.blades.forms {
            for n in 0..FORM_N {
                let s = fm.stab(n);
                if !(0.0..=1.0).contains(&s) {
                    continue;
                }
                self.part(fm.sword_pos(n), CHEST, -fm.dir(n) * 40.0, 0.0, 0.2, 1.4, AQUA, 0.9);
                if s > 0.45 && s < 0.55 {
                    self.ring(fm.center, CHEST, 1.0, 8.0, 0.15, AQUA, 2.0);
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.sprites.retain_mut(|s| {
            s.t += dt;
            sheet(s.kind).0.frame_at(s.t, false).is_some()
        });
        self.parts.retain_mut(|p| {
            p.pos += p.vel * dt;
            p.z = (p.z + p.vz * dt).max(0.0);
            p.vel *= p.drag;
            p.vz *= p.drag;
            p.life -= dt;
            p.life > 0.0
        });
        self.rings.retain_mut(|r| {
            r.t -= dt;
            r.t > 0.0
        });
        self.arcs.retain_mut(|a| {
            a.t -= dt;
            a.t > 0.0
        });
        self.slashes.retain_mut(|s| {
            s.t -= dt;
            s.t > 0.0
        });
        self.ghosts.retain_mut(|g| {
            g.t -= dt;
            g.t > 0.0
        });
        self.nums.retain_mut(|n| {
            n.t -= dt;
            n.t > 0.0
        });
        self.pillars.retain_mut(|p| {
            p.t -= dt;
            p.t > 0.0
        });
        self.spins.retain_mut(|s| {
            s.t -= dt;
            s.rot += dt * if s.big { 34.0 } else { 26.0 };
            s.t > 0.0
        });
        self.shake *= SHAKE_DECAY;
        self.flash = (self.flash - dt).max(0.0);
    }

    fn draw_ring(cam: &Camera, r: &Ring) {
        let k = r.t / r.max;
        let c = cam.air(r.pos, r.z);
        let rr = (r.r1 + (r.r0 - r.r1) * k) * S;
        draw_circle_lines(c.x, c.y, rr, r.w * (0.5 + k), alpha(r.col, k));
    }

    pub fn draw_ground(&self, cam: &Camera) {
        for s in self.sprites.iter().filter(|s| matches!(s.kind, Kind::SerenSigil | Kind::SerenPop)) {
            let (sh, k) = sheet(s.kind);
            if let Some(i) = sh.frame_at(s.t, false) {
                let anchor = if matches!(s.kind, Kind::SerenSigil) { vec2(70.0, 83.0) } else { vec2(56.0, 40.0) };
                sh.draw_at(i, cam.proj(s.pos), anchor, k * S, 1.0);
            }
        }
        self.rings.iter().filter(|r| r.z <= 0.0).for_each(|r| Self::draw_ring(cam, r));
    }

    pub fn draw_ghosts(&self, cam: &Camera, seat: usize, kit: usize) {
        for g in self.ghosts.iter().filter(|g| g.seat == seat) {
            let tex = assets::skin(kit).frame(State::Run, dir8(g.ang), 0, 1);
            let tint = if crate::core::KITS[kit].skin == "warden" { FURY_GHOST } else { GHOST_TINT };
            draw_sprite(tex, cam.proj(g.pos), alpha(tint, (g.t / 0.3).min(1.0) * 0.55));
        }
    }

    pub fn draw_air(&self, cam: &Camera, players: &[Player; 2]) {
        for s in &self.sprites {
            if matches!(s.kind, Kind::SerenSigil | Kind::SerenPop) {
                continue;
            }
            let (sh, k) = sheet(s.kind);
            if matches!(s.kind, Kind::Tempest) {
                let i = (s.t * sh.fps) as usize + 16;
                if i < sh.frames {
                    sh.draw_at(i, cam.proj(s.pos), vec2(76.0, 65.0), k * S, 1.0);
                }
                continue;
            }
            if matches!(s.kind, Kind::SerenLanceHit) {
                let (off, at) = (18, vec2(184.0, 47.0));
                let i = (s.t * sh.fps) as usize + off;
                if i < sh.frames {
                    sh.draw_at(i, cam.air(s.pos, if off == 17 { 2.0 } else { CHEST }), at, k * S, 1.0);
                }
                continue;
            }
            if let Some(i) = sh.frame_at(s.t, false) {
                match s.kind {
                    Kind::Step => sh.draw_rot(i, cam.air(s.pos, CHEST), vec2(20.0, 40.0), k * S, s.ang, 1.0),
                    Kind::Tempest => sh.draw_at(i, cam.proj(s.pos), vec2(74.0, 80.0), k * S, 1.0),
                    Kind::Dash => sh.draw_rot(i, cam.air(s.pos, CHEST), vec2(12.0, 24.0), k * S, s.ang, 1.0),
                    Kind::Blossom => sh.draw_at(i, cam.proj(s.pos), vec2(70.0, 71.0), k * S, 1.0),
                    Kind::Return => sh.draw_at(i, cam.proj(s.pos), vec2(40.0, 60.0), k * S, 1.0),
                    Kind::FoxBlast => sh.draw_at(i, cam.proj(s.pos), vec2(80.0, 47.0), k * S, 1.0),
                    Kind::FoxStrike => sh.draw_at(i, cam.proj(s.pos), vec2(74.0, 44.0), k * S, 1.0),
                    Kind::FoxSwipe => sh.draw_rot(i, cam.air(s.pos, CHEST), vec2(39.0, 31.0), k * S, s.ang, 1.0),
                    Kind::FoxPoof => sh.draw_at(i, cam.air(s.pos, CHEST), vec2(49.0, 44.0), k * S, 1.0),
                    Kind::FoxDash => sh.draw_rot(i, cam.air(s.pos, CHEST), vec2(58.0, 30.0), k * S, s.ang, 1.0),
                    Kind::Bloom => sh.draw(i, cam.air(s.pos, CHEST), k * S, 1.0),
                    Kind::SerenBlast | Kind::SerenBigBlast => sh.draw_at(i, cam.air(s.pos, CHEST), vec2(48.0, 48.0), k * S, 1.0),
                    Kind::SerenThrust => sh.draw_at(i, cam.air(s.pos, CHEST), vec2(32.0, 32.0), k * S, 1.0),
                    _ => sh.draw(i, cam.air(s.pos, 8.0), k * S, 1.0),
                }
            }
        }
        for p in &self.parts {
            let k = p.life / p.max;
            let c = cam.air(p.pos, p.z);
            draw_circle(c.x, c.y, p.size * S * 1.8, alpha(p.col, k * 0.2));
            draw_circle(c.x, c.y, p.size * S * (0.5 + 0.5 * k), alpha(p.col, k));
        }
        for p in &self.pillars {
            let k = p.t / PILLAR_LIFE;
            let g = cam.proj(p.pos);
            let top = g.y - 900.0;
            let w = JUDGE_R * S * 1.3 * k.powf(0.6);
            draw_rectangle(g.x - w, top, w * 2.0, g.y - top, alpha(GOLD, 0.25 * k));
            draw_rectangle(g.x - w * 0.6, top, w * 1.2, g.y - top, alpha(PALE, 0.6 * k));
            draw_rectangle(g.x - w * 0.25, top, w * 0.5, g.y - top, alpha(WHITE, k));
            glow(g, w * 0.8, GOLD, k);
        }
        self.rings.iter().filter(|r| r.z > 0.0).for_each(|r| Self::draw_ring(cam, r));
        for a in &self.arcs {
            let k = a.t / ARC_LIFE;
            let pos = players[a.owner].pos;
            let segs = 14;
            let spread = a.half * 2.0 * (0.4 + 0.6 * (1.0 - k) * 1.6).min(1.0);
            let start = a.ang - a.side * a.half;
            let pts: Vec<Vec2> = (0..=segs).map(|n| cam.air(pos + Vec2::from_angle(start + a.side * spread * n as f32 / segs as f32) * a.r, CHEST)).collect();
            let w = if a.big { 9.0 } else { 6.0 };
            for n in 0..segs {
                let taper = n as f32 / segs as f32;
                let (x, y) = (pts[n], pts[n + 1]);
                draw_line(x.x, x.y, y.x, y.y, w * 2.2 * taper * k, alpha(a.col, 0.25 * k));
                draw_line(x.x, x.y, y.x, y.y, w * taper * k, alpha(a.col, 0.8 * k));
                draw_line(x.x, x.y, y.x, y.y, w * 0.4 * taper * k, alpha(WHITE, k));
            }
            glow(pts[segs], w * 0.6 * k, a.col, k);
        }
        for s in &self.spins {
            let k = s.t / SPIN_LIFE;
            let pos = players[s.owner].pos;
            let r = WHIRL_R * if s.big { 1.15 } else { 1.0 };
            let segs = 18;
            for blade in 0..2 {
                let base = s.rot + blade as f32 * std::f32::consts::PI;
                let col = if blade == 0 { CRIMSON } else { STEEL };
                let pts: Vec<Vec2> = (0..=segs).map(|n| cam.air(pos + Vec2::from_angle(base - 2.4 * n as f32 / segs as f32) * r, CHEST)).collect();
                for n in 0..segs {
                    let taper = 1.0 - n as f32 / segs as f32;
                    let (x, y) = (pts[n], pts[n + 1]);
                    let w = if s.big { 10.0 } else { 7.0 };
                    draw_line(x.x, x.y, y.x, y.y, w * 2.4 * taper * k, alpha(col, 0.22 * k));
                    draw_line(x.x, x.y, y.x, y.y, w * taper * k, alpha(col, 0.85 * k));
                    draw_line(x.x, x.y, y.x, y.y, w * 0.35 * taper * k, alpha(WHITE, k));
                }
                glow(pts[0], 3.0 * S * k, col, k);
            }
        }
        for s in &self.slashes {
            let k = s.t / s.max;
            let (a, b) = (cam.air(s.a, CHEST), cam.air(s.b, CHEST));
            let w = 3.0 * S * k;
            draw_line(a.x, a.y, b.x, b.y, w * 2.4, alpha(s.col, 0.2 * k));
            draw_line(a.x, a.y, b.x, b.y, w, alpha(s.col, 0.75 * k));
            draw_line(a.x, a.y, b.x, b.y, w * 0.35, alpha(WHITE, k));
        }
    }

    pub fn draw_numbers(&self, cam: &Camera) {
        for n in &self.nums {
            let k = n.t / NUM_LIFE;
            let c = cam.air(n.pos, 16.0 + (1.0 - k) * 12.0);
            let base = if n.tag.is_some() { 20.0 } else if n.dmg >= 20 { 40.0 } else { 26.0 };
            let size = base * (1.0 + 0.4 * ((1.0 - k) * 8.0).min(1.0) * k);
            let txt = if n.dmg < 0 { format!("({})", -n.dmg) } else { n.dmg.to_string() };
            let col = n.tag.unwrap_or(if n.dmg >= 30 { GOLD } else if n.dmg >= 20 { PALE } else { WHITE });
            draw_text(&txt, c.x - size * 0.25 + 2.0, c.y + 2.0, size, alpha(BLACK, k));
            draw_text(&txt, c.x - size * 0.25, c.y, size, alpha(col, k));
        }
    }

    pub fn draw_flash(&self) {
        if self.flash > 0.0 {
            draw_rectangle(0.0, 0.0, VIEW.x, VIEW.y, alpha(Color::new(1.0, 0.97, 0.9, 1.0), self.flash * 1.5));
        }
    }
}
