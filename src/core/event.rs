use macroquad::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Frost,
    Holy,
    Blade,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Needle { tip: Vec2, ang: f32 },
    Ward { pos: Vec2, front: Vec2 },
    Blocked { pos: Vec2 },
    RayFired { origin: Vec2, dir: Vec2 },
    JudgeCast { caster: Vec2, at: Vec2 },
    JudgeStrike { pos: Vec2 },
    Cut { owner: usize, ang: f32, r: f32, half: f32, side: f32, big: bool },
    Swallow { pos: Vec2, dir: Vec2 },
    SwallowHit { tip: Vec2 },
    Wind { from: Vec2, to: Vec2, dir: Vec2 },
    Ghost { seat: usize, pos: Vec2, ang: f32, life: f32 },
    FormCast { caster: Vec2, center: Vec2 },
    FormBurst { center: Vec2, base: f32 },
    Hit { pos: Vec2, element: Element, dmg: i32, big: bool },
}
