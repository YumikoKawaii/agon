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

const CLIPS: [&str; 4] = ["idle", "walk", "cast", "hurt"];

pub struct Sprites {
    clips: Vec<Vec<Vec<Texture2D>>>,
}

impl Sprites {
    pub async fn load(root: &str) -> Self {
        let mut clips = Vec::new();
        for name in CLIPS {
            let mut dirs = Vec::new();
            for d in DIRS {
                let mut frames = Vec::new();
                for i in 0.. {
                    let path = format!("{root}/animations/{name}/{d}/frame_{i:03}.png");
                    let Ok(t) = load_texture(&path).await else {
                        break;
                    };
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
