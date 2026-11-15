use std::sync::OnceLock;

use macroquad::texture::{load_texture, FilterMode, Texture2D};

use crate::{
    core::{anim::State, KITS},
    error::{AResult, AgonError},
};

const DIRS: [&str; 8] = ["east", "south-east", "south", "south-west", "west", "north-west", "north", "north-east"];
const CLIPS: [&str; 4] = ["idle", "walk", "cast", "hurt"];
const IDLE_RATE: u32 = 12;
const WALK_RATE: u32 = 7;

pub struct Skin {
    clips: Vec<Vec<Vec<Texture2D>>>,
}

impl Skin {
    async fn load(name: &str) -> AResult<Skin> {
        let mut clips = Vec::new();
        for clip in CLIPS {
            let mut dirs = Vec::new();
            for dir in DIRS {
                let mut frames = Vec::new();
                for i in 0.. {
                    let path = format!("assets/{name}/animations/{clip}/{dir}/frame_{i:03}.png");
                    let Ok(t) = load_texture(&path).await else { break };
                    t.set_filter(FilterMode::Nearest);
                    frames.push(t);
                }
                if frames.is_empty() {
                    return Err(AgonError::Texture { path: format!("assets/{name}/animations/{clip}/{dir}"), source: macroquad::Error::UnknownError("no frames") });
                }
                dirs.push(frames);
            }
            clips.push(dirs);
        }
        Ok(Skin { clips })
    }

    pub fn frame(&self, state: State, dir: usize, age: u32, total: u32) -> &Texture2D {
        let (clip, rate) = match state {
            State::Idle => (0, Some(IDLE_RATE)),
            State::Run | State::Roll => (1, Some(WALK_RATE)),
            State::Attack => (2, None),
            State::Hit => (3, None),
        };
        let frames = &self.clips[clip][dir];
        let n = frames.len() as u32;
        let i = match rate {
            Some(rate) => (age / rate) % n,
            None => (age * n / total.max(1)).min(n - 1),
        };
        &frames[i as usize]
    }
}

pub struct Assets {
    pub skins: Vec<Skin>,
    pub spirit_sword: Texture2D,
}

static INSTANCE: OnceLock<Assets> = OnceLock::new();

async fn texture(path: &str) -> AResult<Texture2D> {
    let t = load_texture(path).await.map_err(|source| AgonError::Texture { path: path.to_string(), source })?;
    t.set_filter(FilterMode::Nearest);
    Ok(t)
}

pub async fn load() -> AResult {
    let mut skins = Vec::new();
    for kit in &KITS {
        skins.push(Skin::load(kit.skin).await?);
    }
    let spirit_sword = texture("assets/sword/spirit_sword.png").await?;
    let _ = INSTANCE.set(Assets { skins, spirit_sword });
    Ok(())
}

pub fn get() -> &'static Assets {
    INSTANCE.get().expect("assets weren't loaded")
}

pub fn skin(kit: usize) -> &'static Skin {
    &get().skins[kit.min(KITS.len() - 1)]
}
