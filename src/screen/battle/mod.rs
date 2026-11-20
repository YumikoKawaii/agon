use macroquad::prelude::*;
use crate::ui::raw::{draw_text, measure_text};

use crate::{
    assets,
    context::Context,
    core::{self, bot::{self, Bot}, Input, DEATHS_TO_WIN, DT},
    net::{
        api::{Endpoint, Seat},
        Link,
    },
    screen::{Screen, StackCommand},
    ui,
};

mod fx;
mod picker;
mod spells;
mod sync;
mod view;

use self::{
    fx::Fx,
    picker::{Action, Bar, Picker},
    sync::Sync,
    view::{letterbox, Camera, VIEW},
};

const BARS: Color = ui::BG;

const PRACTICE_FOE_KIT: usize = 1;

enum Mode {
    Practice(Bot, usize),
    Watch([Bot; 2], [usize; 2]),
    Training,
    Online(Sync),
}

pub struct Battle {
    core: core::Battle,
    fx: Fx,
    cam: Camera,
    mode: Mode,
    pending: Input,
    acc: f32,
    debug: bool,
    canvas: RenderTarget,
    hud: RenderTarget,
    fade: std::cell::Cell<[f32; 3]>,
    lag: std::cell::Cell<[f32; 2]>,
    picker: Picker,
}

const HUD_HIDDEN: f32 = 0.3;

fn read_input(aim: Vec2) -> Input {
    let axis = |neg: KeyCode, pos: KeyCode| is_key_down(pos) as i32 as f32 - is_key_down(neg) as i32 as f32;
    let pressed = [is_mouse_button_pressed(MouseButton::Left), is_key_pressed(KeyCode::Q), is_key_pressed(KeyCode::E), is_key_pressed(KeyCode::R)];
    Input {
        dir: vec2(axis(KeyCode::A, KeyCode::D), axis(KeyCode::W, KeyCode::S)),
        aim,
        slot: (0..4).find(|i| pressed[*i]),
    }
}

impl Battle {
    fn new(core: core::Battle, mode: Mode) -> Self {
        let canvas = render_target(VIEW.x as u32, VIEW.y as u32);
        canvas.texture.set_filter(FilterMode::Nearest);
        let hud = render_target(VIEW.x as u32, VIEW.y as u32);
        hud.texture.set_filter(FilterMode::Nearest);
        Battle { core, fx: Fx::default(), cam: Camera::default(), mode, pending: Input::default(), acc: 0.0, debug: false, canvas, hud, fade: std::cell::Cell::new([1.0; 3]), lag: std::cell::Cell::new([1.0; 2]), picker: Picker::default() }
    }

    pub fn practice(kit: usize) -> Self {
        Self::new(core::Battle::new([kit, PRACTICE_FOE_KIT], 0), Mode::Practice(Bot::new(bot::EASY, 1), 0))
    }

    pub fn watch(kit: usize) -> Self {
        let lv = [1, 1];
        let mut core = core::Battle::new([kit, PRACTICE_FOE_KIT], 0);
        core.stay = true;
        Self::new(core, Mode::Watch([Bot::new(bot::LEVELS[lv[0]].1, 0), Bot::new(bot::LEVELS[lv[1]].1, 1)], lv))
    }

    pub fn training(kit: usize) -> Self {
        Self::new(core::Battle::new([kit, PRACTICE_FOE_KIT], 0), Mode::Training)
    }

    pub fn online(kit: usize, foe: Seat, endpoint: Endpoint) -> Self {
        let link = Link::open(&endpoint.url, &endpoint.ticket, &endpoint.cert_hash);
        let mut core = core::Battle::new([kit, foe.kit], endpoint.seat);
        core.players[1].puppet = true;
        Self::new(core, Mode::Online(Sync::new(link, endpoint.seat, foe.username)))
    }

    fn bar(&self) -> Option<Bar> {
        let kits = [self.core.players[0].kit, self.core.players[1].kit];
        match self.mode {
            Mode::Practice(_, level) => Some(Bar { level: Some(level), watch: None, kits }),
            Mode::Watch(_, lv) => Some(Bar { level: None, watch: Some(lv), kits }),
            Mode::Training => Some(Bar { level: None, watch: None, kits }),
            Mode::Online(_) => None,
        }
    }

    fn restart(&mut self, kits: [usize; 2]) {
        self.core = core::Battle::new(kits, 0);
        self.core.stay = matches!(self.mode, Mode::Watch(..));
        if let Mode::Practice(_, level) = self.mode {
            self.mode = Mode::Practice(Bot::new(bot::LEVELS[level].1, 1), level);
        }
        if let Mode::Watch(_, lv) = self.mode {
            self.mode = Mode::Watch([Bot::new(bot::LEVELS[lv[0]].1, 0), Bot::new(bot::LEVELS[lv[1]].1, 1)], lv);
        }
        self.fx = Fx::default();
        self.lag.set([1.0; 2]);
    }

    fn hud(&self, ctx: &Context) -> view::Hud {
        let me = ctx.user.username.clone();
        match &self.mode {
            Mode::Watch(_, lv) => view::Hud::Equal { tags: [0, 1].map(|i| format!("{} · {}", ["A", "B"][i], bot::LEVELS[lv[i]].0)) },
            Mode::Online(s) => view::Hud::Focus { names: [me, s.foe.clone()], score: Some([s.foe_deaths, self.core.deaths[0]]) },
            Mode::Practice(..) => view::Hud::Focus { names: [me, "Bot".to_string()], score: None },
            Mode::Training => view::Hud::Focus { names: [me, "Dummy".to_string()], score: None },
        }
    }

    fn apply(&mut self, action: Action) {
        let mut kits = [self.core.players[0].kit, self.core.players[1].kit];
        match action {
            Action::Pick(side, kit) => {
                kits[side] = kit;
                self.restart(kits);
            }
            Action::Swap => {
                kits.swap(0, 1);
                self.restart(kits);
            }
            Action::Reset => self.restart(kits),
            Action::Level(side, i) => match &mut self.mode {
                Mode::Practice(bot, level) => {
                    bot.level = bot::LEVELS[i].1;
                    *level = i;
                }
                Mode::Watch(bots, lv) => {
                    bots[side].level = bot::LEVELS[i].1;
                    lv[side] = i;
                }
                _ => {}
            },
        }
    }

    fn step(&mut self) {
        let inputs = match &mut self.mode {
            Mode::Practice(bot, _) => [self.pending, bot.think(&self.core)],
            Mode::Watch(bots, _) => [bots[0].think(&self.core), bots[1].think(&self.core)],
            Mode::Training => [self.pending, Input::default()],
            Mode::Online(sync) => [self.pending, sync.remote_input()],
        };
        self.core.tick(inputs);
        match &mut self.mode {
            Mode::Practice(..) | Mode::Watch(..) => {}
            Mode::Training => {
                let to = self.core.players[0].pos - self.core.players[1].pos;
                self.core.players[1].ang = to.y.atan2(to.x);
            }
            Mode::Online(sync) => {
                sync.correct(&mut self.core);
                let sent = Input { slot: self.core.casted[0], ..self.pending };
                sync.push(&sent, &self.core);
            }
        }
        for e in self.core.drain_events() {
            self.fx.spawn(&e, &self.core.players);
        }
        self.fx.ambient(&self.core);
        self.fx.update(DT);
    }

    fn scoreline(&self, ctx: &Context) -> String {
        match &self.mode {
            Mode::Practice(..) => "vs bot   Esc menu".to_string(),
            Mode::Watch(..) => format!("{} {} - {} {}   Space restart   Esc menu", core::KITS[self.core.players[0].kit].name, self.core.deaths[1], self.core.deaths[0], core::KITS[self.core.players[1].kit].name),
            Mode::Training => "training   Esc menu".to_string(),
            Mode::Online(s) => format!("{} {} - {} {}   first to {} kills   Esc forfeit", ctx.user.username, s.foe_deaths, self.core.deaths[0], s.foe, DEATHS_TO_WIN),
        }
    }
}

impl Screen for Battle {
    fn update(&mut self, _ctx: &mut Context, dt: f32) -> StackCommand {
        let m = self.cam.mouse_view();
        if is_key_pressed(KeyCode::Escape) {
            if self.picker.open.is_some() {
                self.picker.open = None;
            } else {
                return StackCommand::Pop;
            }
        }
        let mut blocked = false;
        if let Some(bar) = self.bar() {
            blocked = self.picker.blocks(m);
            if let Some(a) = self.picker.update(m, &bar) {
                self.apply(a);
            }
        }
        let open = self.picker.open.is_some();
        if !open && is_key_pressed(KeyCode::Tab) {
            self.debug = !self.debug;
        }
        let live = match &mut self.mode {
            Mode::Practice(..) | Mode::Training => !open,
            Mode::Watch(..) => {
                if !open && is_key_pressed(KeyCode::Space) {
                    self.apply(Action::Reset);
                }
                !open
            }
            Mode::Online(sync) => {
                if sync.finished(dt) {
                    return StackCommand::Pop;
                }
                sync.pull();
                sync.live()
            }
        };
        self.cam.shake(self.fx.shake);
        self.cam.follow(&self.core.players, dt);
        let mut r = read_input(self.cam.mouse_world());
        if blocked && r.slot == Some(0) {
            r.slot = None;
        }
        if open {
            r = Input { aim: r.aim, ..Input::default() };
        }
        self.pending.dir = r.dir;
        self.pending.aim = r.aim;
        self.pending.slot = r.slot.or(self.pending.slot);
        self.acc += dt;
        while self.acc >= DT {
            self.acc -= DT;
            if live {
                self.step();
            } else {
                self.fx.update(DT);
            }
            self.pending.slot = None;
        }
        StackCommand::None
    }

    fn draw(&self, ctx: &Context) {
        set_camera(&Camera2D {
            zoom: vec2(2.0 / VIEW.x, 2.0 / VIEW.y),
            target: VIEW / 2.0,
            render_target: Some(self.canvas.clone()),
            ..Default::default()
        });
        clear_background(ui::BG);
        ui::backdrop_in(VIEW.x, VIEW.y);
        self.draw_view(ctx);
        set_default_camera();
        clear_background(BARS);
        ui::backdrop();
        let l = letterbox();
        draw_texture_ex(&self.canvas.texture, l.offset.x, l.offset.y, WHITE, DrawTextureParams { dest_size: Some(VIEW * l.scale), ..Default::default() });
    }

    fn shows_cursor(&self) -> bool {
        false
    }
}

impl Battle {
    fn draw_view(&self, ctx: &Context) {
        let cam = &self.cam;
        let b = &self.core;
        set_camera(&cam.world(&self.canvas));
        view::draw_floor(cam);
        view::draw_cage(cam, false);
        spells::draw_ground(b, cam);
        self.fx.draw_ground(cam);
        let mut order = [0, 1];
        order.sort_by(|x, y| b.players[*x].pos.y.total_cmp(&b.players[*y].pos.y));
        for i in order {
            self.fx.draw_ghosts(cam, i, b.players[i].kit);
            view::draw_player(cam, &b.players[i], self.debug);
        }
        spells::draw_air(b, cam, &assets::get().spirit_sword);
        self.fx.draw_air(cam, &b.players);
        self.fx.draw_numbers(cam);
        view::draw_cage(cam, true);
        let hud = self.hud(ctx);
        let mut lag = self.lag.get();
        for (l, p) in lag.iter_mut().zip(&b.players) {
            *l = view::ease_lag(*l, p, get_frame_time());
        }
        self.lag.set(lag);
        spells::draw_aim(b, cam);
        let line = self.scoreline(ctx);
        let (_, rest) = line.split_once("   ").unwrap_or((line.as_str(), ""));
        let cam_for =|rt: &RenderTarget| Camera2D { zoom: vec2(2.0 / VIEW.x, 2.0 / VIEW.y), target: VIEW / 2.0, render_target: Some(rt.clone()), ..Default::default() };
        set_camera(&cam_for(&self.hud));
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        let m = cam.mouse_view();
        let bar = self.bar();
        view::draw_hud(&b.players, &hud, lag);
        if let Some(bar) = &bar {
            self.picker.draw_bar(m, bar);
        }
        if let Mode::Watch(..) = &self.mode {
            let r = view::score_rect();
            ui::hud_panel(r, 0.45);
            let t = format!("{}  {} - {}  {}", core::KITS[b.players[0].kit].name, b.deaths[1], b.deaths[0], core::KITS[b.players[1].kit].name);
            let w = measure_text(&t, None, 26, 1.0).width;
            draw_text(&t, (VIEW.x - w) / 2.0, r.y + 37.0, 26.0, ui::INK);
        }
        set_camera(&cam_for(&self.canvas));
        self.fx.draw_flash();
        let zones = view::hud_zones(&hud);
        let mut fade = self.fade.get();
        for (z, f) in zones.iter().zip(fade.iter_mut()) {
            let covered = b.players.iter().any(|p| view::body_rect(cam, p).overlaps(z));
            let goal = if covered { HUD_HIDDEN } else { 1.0 };
            *f += (goal - *f) * (get_frame_time() * 10.0).min(1.0);
        }
        self.fade.set(fade);
        for (z, f) in zones.iter().zip(fade) {
            draw_texture_ex(&self.hud.texture, z.x, z.y, Color::new(1.0, 1.0, 1.0, f), DrawTextureParams { source: Some(*z), dest_size: Some(z.size()), ..Default::default() });
        }
        let centered = |t: &str, y: f32, size: u16, c: Color| {
            let w = measure_text(t, None, size, 1.0).width;
            draw_text(t, (VIEW.x - w) / 2.0, y, size as f32, c);
        };
        if let Mode::Online(sync) = &self.mode {
            let banner = sync.banner();
            if let Some(won) = sync.outcome() {
                draw_rectangle(0.0, 0.0, VIEW.x, VIEW.y, Color::new(0.08, 0.06, 0.14, 0.6));
                let r = Rect::new(VIEW.x / 2.0 - 280.0, VIEW.y / 2.0 - 140.0, 560.0, 240.0);
                ui::hud_panel(r, 1.0);
                let (title, col) = if won { ("VICTORY", ui::GOLD) } else { ("DEFEAT", Color::new(0.75, 0.3, 0.4, 1.0)) };
                centered(title, r.y + 110.0, 80, ui::INK);
                centered(title, r.y + 106.0, 80, col);
                centered(&format!("{} {} - {} {}", ctx.user.username, sync.foe_deaths, b.deaths[0], sync.foe), r.y + 160.0, 26, ui::INK);
                centered("returning to lobby...", r.y + 196.0, 18, ui::INK);
            } else if !banner.is_empty() {
                let w = measure_text(&banner, None, 40, 1.0).width + 80.0;
                ui::hud_panel(Rect::new((VIEW.x - w) / 2.0, VIEW.y / 2.0 - 160.0, w, 80.0), 0.5);
                centered(&banner, VIEW.y / 2.0 - 108.0, 40, ui::INK);
            }
        }
        self.picker.draw_panel(m);
        view::draw_cursor(cam);
        let mut y = if let view::Hud::Equal { .. } = hud { VIEW.y - 40.0 } else { 28.0 };
        let keys = if self.bar().is_some() { "WASD move   C characters   Tab boxes" } else { "WASD move   Tab boxes" };
        for part in [keys, rest.trim()] {
            draw_text(part, 20.0, y, 16.0, ui::DIM);
            y += 22.0;
        }
    }
}
