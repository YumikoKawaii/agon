#[derive(Clone, Copy)]
pub struct Circ {
    pub fwd: f32,
    pub side: f32,
    pub r: f32,
}

pub const fn circ(fwd: f32, side: f32, r: f32) -> Circ {
    Circ { fwd, side, r }
}

#[derive(Clone, Copy)]
pub enum Shape {
    Circle(Circ),
    Cone { r: f32, half: f32 },
}

pub struct Frame {
    pub ticks: u32,
    pub hurt: Vec<Circ>,
    pub hit: Vec<Shape>,
}

pub struct Anim {
    pub frames: Vec<Frame>,
    pub looping: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Idle,
    Run,
    Attack,
    Roll,
    Hit,
}

pub const BODY: Circ = circ(0.0, 0.0, 6.0);

fn f(ticks: u32, hurt: &[Circ], hit: &[Shape]) -> Frame {
    Frame {
        ticks,
        hurt: hurt.to_vec(),
        hit: hit.to_vec(),
    }
}

pub struct Anims {
    idle: Anim,
    run: Anim,
    attack: Anim,
    roll: Anim,
    hit: Anim,
}

impl Anims {
    pub fn new() -> Self {
        Anims {
            idle: Anim {
                frames: vec![f(30, &[BODY], &[]), f(30, &[BODY], &[])],
                looping: true,
            },
            run: Anim {
                frames: (0..4).map(|_| f(6, &[BODY], &[])).collect(),
                looping: true,
            },
            attack: Anim {
                frames: vec![
                    f(6, &[BODY], &[]),
                    f(4, &[BODY, circ(12.0, 0.0, 4.0)], &[Shape::Cone { r: 26.0, half: 0.9 }]),
                    f(10, &[BODY, circ(8.0, 0.0, 4.0)], &[]),
                ],
                looping: false,
            },
            roll: Anim {
                frames: vec![
                    f(4, &[], &[]),
                    f(4, &[], &[]),
                    f(4, &[], &[]),
                    f(6, &[circ(0.0, 0.0, 5.0)], &[]),
                ],
                looping: false,
            },
            hit: Anim {
                frames: vec![f(12, &[BODY], &[])],
                looping: false,
            },
        }
    }

    pub fn get(&self, s: State) -> &Anim {
        match s {
            State::Idle => &self.idle,
            State::Run => &self.run,
            State::Attack => &self.attack,
            State::Roll => &self.roll,
            State::Hit => &self.hit,
        }
    }
}
