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
const EXTRA: [&str; 5] = ["transform", "lance", "throw", "punch", "dash"];
const IDLE_RATE: u32 = 12;
const WALK_RATE: u32 = 7;
pub const HD_CELL: f32 = 150.0;
const IDLE_BASE: u32 = 4;
const WALK_BASE: u32 = 6;

pub struct Skin {
    clips: Vec<Vec<Vec<Texture2D>>>,
    extra: Vec<Vec<Vec<Texture2D>>>,
}

const BAR_W: f32 = 360.0;
const BAR_H: f32 = 10.0;

const TOTAL_STEPS: usize = (KITS.len() + 6) * CLIPS.len() * DIRS.len() + 1;

fn draw_progress(done: usize, logo: Option<&Texture2D>, bar: Option<&Nine>) {
    clear_background(ui::BG);
    let cy = screen_height() / 2.0;
    match logo {
        Some(t) => {
            let size = vec2(t.width(), t.height()) * 2.0;
            draw_texture_ex(t, (screen_width() - size.x) / 2.0, cy - 40.0 - size.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        }
        None => ui::text_centered_abs("AGON", cy - 40.0, ui::TITLE, ui::ACCENT),
    }
    let frac = done as f32 / TOTAL_STEPS as f32;
    let r = ui::centered_abs(BAR_W, cy + 10.0, BAR_H * 2.6);
    match bar {
        Some(b) => {
            b.draw(r, r.h / 22.0, WHITE);
            let (px, py) = (r.h * 0.5, r.h * 0.3);
            draw_rectangle(r.x + px, r.y + py, r.w - 2.0 * px, r.h - 2.0 * py, Color::new(0.23, 0.2, 0.31, 1.0));
            draw_rectangle(r.x + px, r.y + py, (r.w - 2.0 * px) * frac, r.h - 2.0 * py, Color::new(0.9, 0.72, 0.36, 1.0));
        }
        None => draw_rectangle(r.x, r.y, r.w * frac, r.h, ui::ACCENT),
    }
    ui::text_centered_abs(&format!("loading {}%", done * 100 / TOTAL_STEPS), r.y + r.h + 30.0, ui::SMALL, ui::DIM);
}

impl Skin {
    async fn load(name: &str, progress: &AtomicUsize) -> AResult<Skin> {
        let mut clips = Vec::new();
        for clip in CLIPS {
            progress.fetch_add(DIRS.len(), Ordering::Relaxed);
            let dirs = sheet_frames(name, clip).await;
            if dirs.iter().any(|d| d.is_empty()) {
                return Err(AgonError::Texture { path: format!("assets/{name}/animations/{clip}"), source: macroquad::Error::UnknownError("no frames") });
            }
            clips.push(dirs);
        }
        let mut extra = Vec::new();
        for clip in EXTRA {
            extra.push(sheet_frames(name, clip).await);
        }
        Ok(Skin { clips, extra })
    }

    pub fn special(&self, clip: &str, dir: usize, age: u32, total: u32) -> Option<&Texture2D> {
        let k = EXTRA.iter().position(|c| *c == clip)?;
        let frames = self.extra[k].get(dir).filter(|f| !f.is_empty())?;
        let n = frames.len() as u32;
        Some(&frames[(age * n / total.max(1)).min(n - 1) as usize])
    }

    pub fn frame(&self, state: State, dir: usize, age: u32, total: u32) -> &Texture2D {
        let (clip, rate) = match state {
            State::Idle => (0, Some(IDLE_RATE * IDLE_BASE / (self.clips[0][dir].len() as u32).max(1))),
            State::Run => (1, Some(WALK_RATE * WALK_BASE / (self.clips[1][dir].len() as u32).max(1))),
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

pub struct ParasolFx {
    pub bolt: Sheet,
    pub bloom: Sheet,
    pub tempest: Sheet,
    pub dash: Sheet,
    pub blossom: Sheet,
    pub ret: Sheet,
    pub top: Sheet,
}

pub struct FoxFx {
    pub orb: Sheet,
    pub wisp: Sheet,
    pub blast: Sheet,
    pub sigil: Sheet,
    pub strike: Sheet,
    pub swipe: Sheet,
    pub poof: Sheet,
    pub dash: Sheet,
}

pub struct SerenFx {
    pub shot: Sheet,
    pub beam: Sheet,
    pub sigil: Sheet,
    pub blast: Sheet,
    pub lance: Sheet,
    pub well: Sheet,
    pub pop: Sheet,
    pub wave: Sheet,
    pub thrust: Sheet,
}

pub struct SwordFx {
    pub wave: Sheet,
    pub aura_back: Sheet,
    pub aura_front: Sheet,
    pub release: Sheet,
}

pub struct Nine {
    pub tex: Texture2D,
    pub src: Rect,
    pub inset: f32,
    pub k: f32,
}

impl Nine {
    async fn load(path: &str, src: Option<Rect>, inset: f32) -> AResult<Nine> {
        Self::load_k(path, src, inset, 1.0).await
    }

    async fn load_k(path: &str, src: Option<Rect>, inset: f32, k: f32) -> AResult<Nine> {
        let tex = texture(path).await?;
        let src = src.unwrap_or(Rect::new(0.0, 0.0, tex.width(), tex.height()));
        Ok(Nine { tex, src, inset, k })
    }

    pub fn draw(&self, r: Rect, scale: f32, tint: Color) {
        let (s, i) = (self.src, self.inset);
        let d = (i * scale * self.k).round();
        let xs = [(s.x, i, r.x, d), (s.x + i, s.w - 2.0 * i, r.x + d, r.w - 2.0 * d), (s.x + s.w - i, i, r.x + r.w - d, d)];
        let ys = [(s.y, i, r.y, d), (s.y + i, s.h - 2.0 * i, r.y + d, r.h - 2.0 * d), (s.y + s.h - i, i, r.y + r.h - d, d)];
        for (sx, sw, dx, dw) in xs {
            for (sy, sh, dy, dh) in ys {
                if dw <= 0.0 || dh <= 0.0 {
                    continue;
                }
                draw_texture_ex(&self.tex, dx, dy, tint, DrawTextureParams { source: Some(Rect::new(sx, sy, sw, sh)), dest_size: Some(vec2(dw, dh)), ..Default::default() });
            }
        }
    }
}

pub struct UiKit {
    pub panel: Nine,
    pub hud: Nine,
    pub arena: Texture2D,
    pub tiles: Vec<Texture2D>,
    pub button: Nine,
    pub field: Nine,
    pub bar: Nine,
    pub logo: Texture2D,
    pub reticle: Texture2D,
    pub unknown: Texture2D,
    pub versus: Texture2D,
    pub modes: [Texture2D; 3],
    pub icons: Vec<Texture2D>,
}

pub const ICONS: [&str; 41] = [
    "needle", "ray", "ward", "judgement", "cut", "swallow", "windstep", "formation", "hex", "potion", "broom", "cauldron", "grace", "hexed", "intent", "paperbond", "talisman", "parasol", "petaldash", "blossom", "spiritreturn", "tempest",
    "foxorb", "unpredictable", "shininglight", "blindinglight", "foxclaw", "foxswipe", "foxleap", "foxball", "foxreturn", "foxform",
    "manashot", "azurelance", "gravitywell", "overdrive", "runeknuckle", "overfists", "rocketdash", "starbreaker", "overcharge",
];
pub const PASSIVE_ICON: usize = 12;
const PARASOL_ICON: usize = 16;
const FOX_ICON: usize = 22;
const SEREN_ICON: usize = 32;

pub fn skill_icon(s: crate::core::Skill) -> Option<&'static Texture2D> {
    use crate::core::Skill;
    let i = match s {
        Skill::Talisman => PARASOL_ICON,
        Skill::Parasol => PARASOL_ICON + 1,
        Skill::PetalDash => PARASOL_ICON + 2,
        Skill::Blossom => PARASOL_ICON + 3,
        Skill::SpiritStep => PARASOL_ICON + 4,
        Skill::Storm => PARASOL_ICON + 5,
        Skill::FoxOrb => FOX_ICON,
        Skill::Unpredictable => FOX_ICON + 1,
        Skill::ShiningLight => FOX_ICON + 2,
        Skill::BlindLight => FOX_ICON + 3,
        Skill::FoxClaw => FOX_ICON + 4,
        Skill::FoxSwipe => FOX_ICON + 5,
        Skill::FoxWisp => FOX_ICON + 6,
        Skill::Vanish => FOX_ICON + 8,
        Skill::ManaShot => SEREN_ICON,
        Skill::AzureLance => SEREN_ICON + 1,
        Skill::GravityWell => SEREN_ICON + 2,
        Skill::Overdrive => SEREN_ICON + 3,
        Skill::RuneKnuckle => SEREN_ICON + 4,
        Skill::OverFists => SEREN_ICON + 5,
        Skill::RocketDash => SEREN_ICON + 6,
        Skill::Starbreaker => SEREN_ICON + 7,
        _ => s as usize,
    };
    (i < PASSIVE_ICON || (PARASOL_ICON..ICONS.len()).contains(&i)).then(|| &get().ui.icons[i])
}

const ZOOMED_ICONS: [usize; 6] = [SEREN_ICON, SEREN_ICON + 1, SEREN_ICON + 2, SEREN_ICON + 3, SEREN_ICON + 7, SEREN_ICON + 8];

pub fn zoomed_icon(t: &Texture2D) -> bool {
    let icons = &get().ui.icons;
    ZOOMED_ICONS.iter().any(|i| std::ptr::eq(t, &icons[*i]))
}

pub fn passive_icon(p: crate::core::kit::Passive) -> &'static Texture2D {
    match p {
        crate::core::kit::Passive::Fox => &get().ui.icons[FOX_ICON + 9],
        crate::core::kit::Passive::Overcharge => &get().ui.icons[SEREN_ICON + 8],
        _ => &get().ui.icons[PASSIVE_ICON + p as usize],
    }
}

pub fn portrait(kit: usize) -> &'static Texture2D {
    &get().portraits[kit.min(KITS.len() - 1)]
}

pub struct Assets {
    pub ui: UiKit,
    pub portraits: Vec<Texture2D>,
    pub skins: Vec<Skin>,
    pub unarmed: Skin,
    pub fox_form: Skin,
    pub seren_form: Skin,
    pub seren_lit: Skin,
    pub seren_charged: Skin,
    pub seren_fading: Skin,
    pub seren: SerenFx,
    pub spirit_sword: Texture2D,
    pub witch: WitchFx,
    pub sword: SwordFx,
    pub parasol: ParasolFx,
    pub fox: FoxFx,
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
    let unarmed = Skin::load("parasol_unarmed", progress).await?;
    let fox_form = Skin::load("fox_form", progress).await?;
    let seren_form = Skin::load("seren_form", progress).await?;
    let seren_lit = Skin::load("seren_lit", progress).await?;
    let seren_charged = Skin::load("seren_charged", progress).await?;
    let seren_fading = Skin::load("seren_fading", progress).await?;
    let seren = SerenFx {
        shot: Sheet::load("assets/seren/fx/seren-mana-shot.png", 48.0, 48.0, 8, 12.0).await?,
        beam: Sheet::load("assets/seren/fx/seren-starbreaker-beam.png", 297.0, 84.0, 19, 15.0).await?,
        sigil: Sheet::load("assets/seren/fx/seren-transform-sigil.png", 140.0, 112.0, 16, 16.0).await?,
        blast: Sheet::load("assets/seren/fx/seren-arcane-blast.png", 96.0, 96.0, 10, 14.0).await?,
        lance: Sheet::load("assets/seren/fx/seren-azure-lance.png", 246.0, 116.0, 31, 30.0).await?,
        well: Sheet::load("assets/seren/fx/seren-gravity-well.png", 128.0, 112.0, 12, 12.0).await?,
        pop: Sheet::load("assets/seren/fx/seren-well-pop.png", 112.0, 64.0, 7, 14.0).await?,
        wave: Sheet::load("assets/seren/fx/seren-punch-wave.png", 96.0, 96.0, 7, 16.0).await?,
        thrust: Sheet::load("assets/seren/fx/seren-thruster.png", 64.0, 64.0, 6, 12.0).await?,
    };
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
    let parasol = ParasolFx {
        bolt: Sheet::load("assets/parasol/fx_v3/talisman-bolt.png", 54.0, 25.0, 8, 12.0).await?,
        dash: Sheet::load("assets/parasol/fx_v3/petal-dash.png", 111.0, 42.0, 10, 16.0).await?,
        blossom: Sheet::load("assets/parasol/fx_v3/blossom-spin.png", 157.0, 73.0, 12, 16.0).await?,
        top: Sheet::load("assets/parasol/fx_v3/parasol-top.png", 64.0, 64.0, 1, 1.0).await?,
        ret: Sheet::load("assets/parasol/fx_v3/spirit-return.png", 96.0, 104.0, 10, 16.0).await?,
        bloom: Sheet::load("assets/parasol/fx_v3/bloom-pop.png", 48.0, 49.0, 8, 16.0).await?,
        tempest: Sheet::load("assets/parasol/fx_v3/storm-burst.png", 171.0, 106.0, 16, 16.0).await?,
    };
    let fox = FoxFx {
        orb: Sheet::load("assets/fox/fx/fox-orb-v2.png", 61.0, 39.0, 24, 12.0).await?,
        wisp: Sheet::load("assets/fox/fx/fox-wisp.png", 69.0, 34.0, 12, 12.0).await?,
        blast: Sheet::load("assets/fox/fx/fox-blinding-blast-v2.png", 160.0, 109.0, 12, 16.0).await?,
        sigil: Sheet::load("assets/fox/fx/fox-spirit-sigil.png", 144.0, 72.0, 24, 12.0).await?,
        strike: Sheet::load("assets/fox/fx/fox-foxfire-flare.png", 148.0, 100.0, 12, 16.0).await?,
        swipe: Sheet::load("assets/fox/fx/fox-swipe-v2.png", 76.0, 66.0, 8, 16.0).await?,
        poof: Sheet::load("assets/fox/fx/fox-transform-v2.png", 93.0, 103.0, 14, 16.0).await?,
        dash: Sheet::load("assets/fox/fx/fox-dash-v2.png", 134.0, 60.0, 12, 16.0).await?,
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
    let ui = UiKit {
        panel: Nine::load_k("assets/ui/kit.png", Some(Rect::new(182.0, 272.0, 147.0, 72.0)), 26.0, 2.8).await?,
        arena: texture("assets/ui/arena.png").await?,
        tiles: {
            let mut v = Vec::new();
            for i in 0..16 {
                v.push(texture(&format!("assets/tiles/dark/tile_{i}.png")).await?);
            }
            v
        },
        hud: Nine::load("assets/ui/panel.png", None, 28.0).await?,
        button: Nine::load_k("assets/ui/kit.png", Some(Rect::new(225.0, 243.0, 60.0, 23.0)), 7.0, 1.8).await?,
        field: Nine::load("assets/ui/field.png", Some(Rect::new(4.0, 4.0, 62.0, 22.0)), 6.0).await?,
        bar: Nine::load("assets/ui/bars.png", None, 10.0).await?,
        logo: texture("assets/ui/logo.png").await?,
        reticle: texture("assets/ui/reticle.png").await?,
        unknown: texture("assets/ui/unknown.png").await?,
        versus: texture("assets/ui/versus.png").await?,
        modes: [texture("assets/ui/mode_bot.png").await?, texture("assets/ui/mode_training.png").await?, texture("assets/ui/mode_watch.png").await?],
        icons: {
            let mut v = Vec::new();
            for n in ICONS {
                v.push(texture(&format!("assets/ui/skills/{n}.png")).await?);
            }
            v
        },
    };
    let mut portraits = Vec::new();
    for kit in &KITS {
        portraits.push(texture(&format!("assets/{}/portrait.png", kit.skin)).await?);
    }
    Ok(Assets { ui, portraits, skins, unarmed, fox_form, seren_form, seren_lit, seren_charged, seren_fading, seren, spirit_sword, witch, sword, parasol, fox, status, bleed_pop })
}

pub async fn load() -> AResult {
    let logo = texture("assets/ui/logo.png").await.ok();
    let bar = Nine::load("assets/ui/bars.png", None, 10.0).await.ok();
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
        draw_progress(progress.load(Ordering::Relaxed), logo.as_ref(), bar.as_ref());
        next_frame().await;
    }
}

pub fn get() -> &'static Assets {
    INSTANCE.get().expect("assets weren't loaded")
}

pub fn skin(kit: usize) -> &'static Skin {
    &get().skins[kit.min(KITS.len() - 1)]
}

pub fn skin_in(kit: usize, armed: bool) -> &'static Skin {
    if !armed && KITS[kit.min(KITS.len() - 1)].skin == "parasol" {
        &get().unarmed
    } else if !armed && KITS[kit.min(KITS.len() - 1)].skin == "fox" {
        &get().fox_form
    } else if !armed && KITS[kit.min(KITS.len() - 1)].skin == "seren" {
        &get().seren_form
    } else {
        skin(kit)
    }
}

async fn sheet_frames(name: &str, clip: &str) -> Vec<Vec<Texture2D>> {
    let base = format!("assets/{name}/animations/{clip}");
    let Ok(meta) = load_string(&format!("{base}/sheet.txt")).await else {
        return vec![Vec::new(); DIRS.len()];
    };
    let Ok(img) = load_image(&format!("{base}/sheet.png")).await else {
        return vec![Vec::new(); DIRS.len()];
    };
    let nums: Vec<usize> = meta.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    let (cw, ch) = (nums[0] as f32, nums[1] as f32);
    nums[2..]
        .iter()
        .enumerate()
        .map(|(r, &n)| {
            (0..n)
                .map(|c| {
                    let t = Texture2D::from_image(&img.sub_image(Rect::new(c as f32 * cw, r as f32 * ch, cw, ch)));
                    t.set_filter(FilterMode::Nearest);
                    t
                })
                .collect()
        })
        .collect()
}
