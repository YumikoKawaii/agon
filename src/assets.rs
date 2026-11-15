use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};

use macroquad::{experimental::coroutines::start_coroutine, prelude::*};

use crate::{
    core::{anim::State, KITS},
    error::{AResult, AgonError},
    ui,
};

const DIRS: [&str; 8] = ["east", "south-east", "south", "south-west", "west", "north-west", "north", "north-east"];
const CLIPS: [&str; 4] = ["idle", "walk", "cast", "hurt"];
const IDLE_RATE: u32 = 12;
const WALK_RATE: u32 = 7;

pub struct Skin {
    clips: Vec<Vec<Vec<Texture2D>>>,
}

const BAR_W: f32 = 360.0;
const BAR_H: f32 = 10.0;

const TOTAL_STEPS: usize = KITS.len() * CLIPS.len() * DIRS.len() + 1;

fn draw_progress(done: usize) {
    clear_background(ui::BG);
    ui::text_centered("AGON", screen_height() / 2.0 - 40.0, ui::TITLE, ui::ACCENT);
    let r = ui::centered(BAR_W, screen_height() / 2.0 + 10.0, BAR_H);
    draw_rectangle(r.x, r.y, r.w, r.h, ui::PANEL);
    draw_rectangle(r.x, r.y, r.w * done as f32 / TOTAL_STEPS as f32, r.h, ui::ACCENT);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, ui::EDGE);
    ui::text_centered(&format!("loading {}%", done * 100 / TOTAL_STEPS), r.y + 40.0, ui::SMALL, ui::DIM);
}

impl Skin {
    async fn load(name: &str, progress: &AtomicUsize) -> AResult<Skin> {
        let mut clips = Vec::new();
        for clip in CLIPS {
            let mut dirs = Vec::new();
            for dir in DIRS {
                progress.fetch_add(1, Ordering::Relaxed);
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

async fn load_all(progress: &AtomicUsize) -> AResult<Assets> {
    let mut skins = Vec::new();
    for kit in &KITS {
        skins.push(Skin::load(kit.skin, progress).await?);
    }
    progress.fetch_add(1, Ordering::Relaxed);
    let spirit_sword = texture("assets/sword/spirit_sword.png").await?;
    Ok(Assets { skins, spirit_sword })
}

pub async fn load() -> AResult {
    let progress = Arc::new(AtomicUsize::new(0));
    let result = Arc::new(Mutex::new(None));
    let (p, r) = (progress.clone(), result.clone());
    start_coroutine(async move {
        let loaded = load_all(&p).await;
        *r.lock().expect("asset result lock") = Some(loaded);
    });
    loop {
        let loaded = result.lock().expect("asset result lock").take();
        if let Some(loaded) = loaded {
            let _ = INSTANCE.set(loaded?);
            return Ok(());
        }
        draw_progress(progress.load(Ordering::Relaxed));
        next_frame().await;
    }
}

pub fn get() -> &'static Assets {
    INSTANCE.get().expect("assets weren't loaded")
}

pub fn skin(kit: usize) -> &'static Skin {
    &get().skins[kit.min(KITS.len() - 1)]
}
