#[derive(Clone, Copy)]
pub enum Shape {
    Circle { fwd: f32, side: f32, r: f32 },
    Cone { r: f32, half: f32 },
}

pub struct Frame {
    pub ticks: u32,
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
    Hit,
}

fn f(ticks: u32, hit: &[Shape]) -> Frame {
    Frame {
        ticks,
        hit: hit.to_vec(),
    }
}

pub struct Anims {
    idle: Anim,
    run: Anim,
    attack: Anim,
    hit: Anim,
}

impl Anims {
    pub fn new() -> Self {
        Anims {
            idle: Anim {
                frames: vec![f(30, &[]), f(30, &[])],
                looping: true,
            },
            run: Anim {
                frames: (0..4).map(|_| f(6, &[])).collect(),
                looping: true,
            },
            attack: Anim {
                frames: vec![
                    f(6, &[]),
                    f(4, &[Shape::Cone { r: 26.0, half: 0.9 }]),
                    f(10, &[]),
                ],
                looping: false,
            },
            hit: Anim {
                frames: vec![f(12, &[])],
                looping: false,
            },
        }
    }

    pub fn get(&self, s: State) -> &Anim {
        match s {
            State::Idle => &self.idle,
            State::Run => &self.run,
            State::Attack => &self.attack,
            State::Hit => &self.hit,
        }
    }
}
