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

pub struct Sheet {
    tex: Texture2D,
    w: f32,
    h: f32,
    cols: usize,
    pub frames: usize,
    pub fps: f32,
}

impl Sheet {
    async fn load(path: &str, w: f32, h: f32, frames: usize, fps: f32) -> AResult<Sheet> {
        let tex = texture(path).await?;
        let cols = (tex.width() / w) as usize;
        Ok(Sheet { tex, w, h, cols, frames, fps })
    }

    pub fn frame_at(&self, t: f32, looping: bool) -> Option<usize> {
        let i = (t * self.fps) as usize;
        if looping {
            Some(i % self.frames)
        } else if i < self.frames {
            Some(i)
        } else {
            None
        }
    }

    pub fn draw_at(&self, i: usize, at: Vec2, anchor: Vec2, scale: f32, a: f32) {
        let centre = vec2(self.w, self.h) / 2.0;
        self.draw(i, at + (centre - anchor) * scale, scale, a);
    }

    pub fn draw_rot(&self, i: usize, at: Vec2, pivot: Vec2, scale: f32, ang: f32, a: f32) {
        let src = Rect::new((i % self.cols) as f32 * self.w, (i / self.cols) as f32 * self.h, self.w, self.h);
        let size = vec2(self.w, self.h) * scale;
        let tl = at - pivot * scale;
        draw_texture_ex(&self.tex, tl.x, tl.y, Color::new(1.0, 1.0, 1.0, a), DrawTextureParams { source: Some(src), dest_size: Some(size), rotation: ang, pivot: Some(at), ..Default::default() });
    }

    pub fn draw(&self, i: usize, at: Vec2, scale: f32, a: f32) {
        let src = Rect::new((i % self.cols) as f32 * self.w, (i / self.cols) as f32 * self.h, self.w, self.h);
        let size = vec2(self.w, self.h) * scale;
        draw_texture_ex(&self.tex, at.x - size.x / 2.0, at.y - size.y / 2.0, Color::new(1.0, 1.0, 1.0, a), DrawTextureParams { source: Some(src), dest_size: Some(size), ..Default::default() });
    }
}

pub struct WitchFx {
    pub bolt: Sheet,
    pub potion: Sheet,
    pub burst: Sheet,
    pub sigil: Sheet,
    pub poof: Sheet,
}

pub struct Status {
    pub sheet: Sheet,
    pub anchor: Vec2,
    pub ground: bool,
}

pub struct SwordFx {
    pub wave: Sheet,
    pub aura_back: Sheet,
    pub aura_front: Sheet,
    pub release: Sheet,
}

pub struct Assets {
    pub skins: Vec<Skin>,
    pub spirit_sword: Texture2D,
    pub witch: WitchFx,
    pub sword: SwordFx,
    pub status: Vec<Status>,
    pub bleed_pop: Status,
}

async fn load_status(name: &str, w: f32, h: f32, frames: usize, fps: f32, anchor: Vec2, ground: bool) -> AResult<Status> {
    let sheet = Sheet::load(&format!("assets/fx/{name}.png"), w, h, frames, fps).await?;
    Ok(Status { sheet, anchor, ground })
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
    let witch = WitchFx {
        bolt: Sheet::load("assets/witch/fx/hex_bolt.png", 48.0, 48.0, 8, 12.0).await?,
        potion: Sheet::load("assets/witch/fx/potion_burst.png", 76.0, 80.0, 8, 12.0).await?,
        burst: Sheet::load("assets/witch/fx/cauldron_burst.png", 118.0, 129.0, 8, 12.0).await?,
        sigil: Sheet::load("assets/witch/fx/cauldron_sigil.png", 120.0, 123.0, 12, 12.0).await?,
        poof: Sheet::load("assets/witch/fx/broom_poof.png", 62.0, 62.0, 6, 12.0).await?,
    };
    let sword = SwordFx {
        wave: Sheet::load("assets/sword/fx/intent/part-wave_slash.png", 103.0, 49.0, 13, 16.0).await?,
        aura_back: Sheet::load("assets/sword/fx/aura/behind.png", 52.0, 49.0, 24, 12.0).await?,
        aura_front: Sheet::load("assets/sword/fx/aura/front.png", 54.0, 64.0, 24, 12.0).await?,
        release: Sheet::load("assets/sword/fx/release/sword-intent-release.png", 64.0, 51.0, 8, 16.0).await?,
    };
    let status = vec![
        load_status("slow", 36.0, 43.0, 12, 10.0, vec2(17.0, 34.0), true).await?,
        load_status("root", 64.0, 58.0, 22, 14.0, vec2(32.0, 44.0), true).await?,
        load_status("bleed", 29.0, 41.0, 24, 12.0, vec2(14.0, 4.0), false).await?,
        load_status("burn", 54.0, 57.0, 12, 12.0, vec2(27.0, 46.0), true).await?,
        load_status("mark", 61.0, 31.0, 24, 12.0, vec2(30.0, 13.0), false).await?,
        load_status("shield", 61.0, 61.0, 12, 10.0, vec2(30.0, 30.0), false).await?,
        load_status("vulnerable", 46.0, 52.0, 16, 10.0, vec2(24.0, 37.0), false).await?,
        load_status("stun", 39.0, 23.0, 12, 12.0, vec2(19.0, 15.0), false).await?,
    ];
    let bleed_pop = load_status("bleed_pop", 66.0, 69.0, 8, 16.0, vec2(35.0, 30.0), false).await?;
    Ok(Assets { skins, spirit_sword, witch, sword, status, bleed_pop })
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
