pub mod anim;
pub mod battle;
pub mod bot;
pub mod effect;
pub mod fox;
pub mod event;
pub mod hex;
pub mod kit;
pub mod magic;
pub mod parasol;
pub mod player;
pub mod seren;
pub mod sword;

pub use battle::Battle;
pub use kit::{Skill, KITS};
pub use player::{Input, Player};

use macroquad::math::{vec2, Vec2};

pub const ARENA: Vec2 = vec2(2400.0, 1400.0);
pub const RADIUS: f32 = 6.0;
pub const DT: f32 = 1.0 / 60.0;
pub const DEATHS_TO_WIN: u32 = 3;

pub fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(0.0001)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

pub fn inside(p: Vec2, margin: f32) -> Vec2 {
    p.clamp(Vec2::splat(margin), ARENA - margin)
}
