#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Idle,
    Run,
    Attack,
    Hit,
}

pub struct Frame {
    pub ticks: u32,
    pub hurt: Option<f32>,
}

pub struct Anim {
    pub frames: &'static [Frame],
    pub looping: bool,
}

const fn f(ticks: u32, hurt: Option<f32>) -> Frame {
    Frame { ticks, hurt }
}

const BODY: Option<f32> = Some(6.0);

const IDLE: Anim = Anim { frames: &[f(30, BODY), f(30, BODY)], looping: true };
const RUN: Anim = Anim { frames: &[f(6, BODY), f(6, BODY), f(6, BODY), f(6, BODY)], looping: true };
const ATTACK: Anim = Anim { frames: &[f(4, BODY), f(4, BODY), f(10, BODY)], looping: false };
const HIT: Anim = Anim { frames: &[f(12, BODY)], looping: false };

impl State {
    pub fn anim(self) -> &'static Anim {
        match self {
            State::Idle => &IDLE,
            State::Run => &RUN,
            State::Attack => &ATTACK,
            State::Hit => &HIT,
        }
    }
}
