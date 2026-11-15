use crate::anim::State;
use macroquad::prelude::*;

const DIRS: [&str; 8] = [
    "east",
    "south-east",
    "south",
    "south-west",
    "west",
    "north-west",
    "north",
    "north-east",
];

const CLIPS: [(&str, usize); 4] = [("idle", 4), ("walk", 6), ("cast", 7), ("hurt", 5)];

pub struct Sprites {
    clips: Vec<Vec<Vec<Texture2D>>>,
}

impl Sprites {
    pub async fn load(root: &str) -> Self {
        let mut clips = Vec::new();
        for (name, n) in CLIPS {
            let mut dirs = Vec::new();
            for d in DIRS {
                let mut frames = Vec::new();
                for i in 0..n {
                    let path = format!("{root}/animations/{name}/{d}/frame_{i:03}.png");
                    let t = load_texture(&path).await.unwrap();
                    t.set_filter(FilterMode::Nearest);
                    frames.push(t);
                }
                dirs.push(frames);
            }
            clips.push(dirs);
        }
        Sprites { clips }
    }

    pub fn frame(&self, state: State, dir: i32, age: u32, total: u32) -> &Texture2D {
        let (clip, looping, rate) = match state {
            State::Idle => (0, true, 12),
            State::Run | State::Roll => (1, true, 7),
            State::Attack => (2, false, 0),
            State::Hit => (3, false, 0),
        };
        let frames = &self.clips[clip][dir as usize];
        let n = frames.len() as u32;
        let i = if looping {
            (age / rate) % n
        } else {
            (age * n / total.max(1)).min(n - 1)
        };
        &frames[i as usize]
    }
}
