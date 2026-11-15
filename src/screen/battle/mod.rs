use macroquad::prelude::*;

use crate::{
    assets,
    context::Context,
    core::{self, bot::{self, Bot}, Input, DEATHS_TO_WIN, DT, KITS},
    net::{
        api::{Endpoint, Seat},
        Link,
    },
    screen::{Screen, StackCommand},
    ui,
};

mod fx;
mod spells;
mod sync;
mod view;

use self::{
    fx::Fx,
    sync::Sync,
    view::{letterbox, Camera, VIEW},
};

const BARS: Color = Color::new(0.0, 0.0, 0.0, 1.0);

const PRACTICE_FOE_KIT: usize = 1;
const LEVEL_KEYS: [KeyCode; 3] = [KeyCode::Key4, KeyCode::Key5, KeyCode::Key6];
const KIT_KEYS: [KeyCode; 3] = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3];

enum Mode {
    Practice(Bot, usize),
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
}

fn read_input(aim: Vec2) -> Input {
    let axis = |neg: KeyCode, pos: KeyCode| is_key_down(pos) as i32 as f32 - is_key_down(neg) as i32 as f32;
    let pressed = [is_mouse_button_pressed(MouseButton::Left), is_key_pressed(KeyCode::Q), is_key_pressed(KeyCode::E), is_key_pressed(KeyCode::R)];
    Input {
        dir: vec2(axis(KeyCode::A, KeyCode::D), axis(KeyCode::W, KeyCode::S)),
        aim,
        slot: (0..4).find(|i| pressed[*i]),
        dodge: is_mouse_button_pressed(MouseButton::Right),
    }
}

impl Battle {
    fn new(core: core::Battle, mode: Mode) -> Self {
        let canvas = render_target(VIEW.x as u32, VIEW.y as u32);
        canvas.texture.set_filter(FilterMode::Nearest);
        Battle { core, fx: Fx::default(), cam: Camera::default(), mode, pending: Input::default(), acc: 0.0, debug: false, canvas }
    }

    pub fn practice(kit: usize) -> Self {
        Self::new(core::Battle::new([kit, PRACTICE_FOE_KIT], 0), Mode::Practice(Bot::new(bot::EASY, 1), 0))
    }

    pub fn training(kit: usize) -> Self {
        Self::new(core::Battle::new([kit, PRACTICE_FOE_KIT], 0), Mode::Training)
    }

    pub fn online(kit: usize, foe: Seat, endpoint: Endpoint) -> Self {
        let link = Link::open(&endpoint.url, &endpoint.ticket, &endpoint.cert_hash);
        let core = core::Battle::new([kit, foe.kit], endpoint.seat);
        Self::new(core, Mode::Online(Sync::new(link, endpoint.seat, foe.username)))
    }

    fn switch_kit(&mut self) {
        let level = match self.mode {
            Mode::Practice(_, level) => Some(level),
            Mode::Training => None,
            Mode::Online(_) => return,
        };
        let Some(kit) = KIT_KEYS.iter().position(|k| is_key_pressed(*k)) else { return };
        let foe = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let mut kits = [self.core.players[0].kit, self.core.players[1].kit];
        if kits[foe as usize] == kit {
            return;
        }
        kits[foe as usize] = kit;
        self.core = core::Battle::new(kits, 0);
        if let Some(level) = level {
            self.mode = Mode::Practice(Bot::new(bot::LEVELS[level].1, 1), level);
        }
        self.fx = Fx::default();
    }

    fn switch_level(&mut self) {
        let Mode::Practice(bot, level) = &mut self.mode else { return };
        if let Some(i) = LEVEL_KEYS.iter().position(|k| is_key_pressed(*k)) {
            bot.level = bot::LEVELS[i].1;
            *level = i;
        }
    }

    fn step(&mut self) {
        let foe_input = match &mut self.mode {
            Mode::Practice(bot, _) => bot.think(&self.core),
            Mode::Training => Input::default(),
            Mode::Online(sync) => sync.remote_input(),
        };
        self.core.tick([self.pending, foe_input]);
        match &mut self.mode {
            Mode::Practice(..) => {}
            Mode::Training => {
                let to = self.core.players[0].pos - self.core.players[1].pos;
                self.core.players[1].ang = to.y.atan2(to.x);
            }
            Mode::Online(sync) => {
                sync.correct(&mut self.core.players[1]);
                sync.push(&self.pending, &self.core.players[0], self.core.deaths[0]);
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
            Mode::Practice(_, level) => format!("vs bot   1/2/3 character   Shift+1/2/3 bot   4/5/6 difficulty: {}   Esc menu", bot::LEVELS[*level].0),
            Mode::Training => "training   1/2/3 character   Shift+1/2/3 dummy   Esc menu".to_string(),
            Mode::Online(s) => format!("{} {} - {} {}   first to {} kills   Esc forfeit", ctx.user.username, s.foe_deaths, self.core.deaths[0], s.foe, DEATHS_TO_WIN),
        }
    }
}

impl Screen for Battle {
    fn update(&mut self, _ctx: &mut Context, dt: f32) -> StackCommand {
        if is_key_pressed(KeyCode::Escape) {
            return StackCommand::Pop;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.debug = !self.debug;
        }
        self.switch_kit();
        self.switch_level();
        let live = match &mut self.mode {
            Mode::Practice(..) | Mode::Training => true,
            Mode::Online(sync) => {
                if sync.finished(dt) {
                    return StackCommand::Pop;
                }
                sync.pull();
                sync.live()
            }
        };
        self.cam.shake(self.fx.shake);
        let r = read_input(self.cam.mouse_world());
        self.pending.dir = r.dir;
        self.pending.aim = r.aim;
        self.pending.slot = r.slot.or(self.pending.slot);
        self.pending.dodge |= r.dodge;
        self.acc += dt;
        while self.acc >= DT {
            self.acc -= DT;
            if live {
                self.step();
            } else {
                self.fx.update(DT);
            }
            self.pending.slot = None;
            self.pending.dodge = false;
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
        self.draw_view(ctx);
        set_default_camera();
        clear_background(BARS);
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
        spells::draw_aim(b, cam);
        view::draw_cursor(cam);
        self.fx.draw_flash();
        view::draw_hud(&b.players);
        let centered = |t: &str, y: f32, size: u16, c: Color| {
            let w = measure_text(t, None, size, 1.0).width;
            draw_text(t, (VIEW.x - w) / 2.0, y, size as f32, c);
        };
        centered(&self.scoreline(ctx), 20.0, 20, LIGHTGRAY);
        if let Mode::Online(sync) = &self.mode {
            centered(&sync.banner(), VIEW.y / 2.0 - 120.0, 64, WHITE);
        }
        draw_text("WASD move   Mouse aim   RMB roll   Tab boxes", 40.0, VIEW.y - 34.0, 22.0, LIGHTGRAY);
        draw_text(KITS[b.players[0].kit].hint, 40.0, VIEW.y - 10.0, 22.0, LIGHTGRAY);
    }
}
