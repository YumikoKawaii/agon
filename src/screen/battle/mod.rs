use macroquad::prelude::*;

use crate::{
    assets,
    context::Context,
    core::{self, Input, DEATHS_TO_WIN, DT, KITS},
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

use self::{fx::Fx, sync::Sync, view::Camera};

const PRACTICE_FOE_KIT: usize = 1;
const KIT_KEYS: [KeyCode; 3] = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3];

enum Mode {
    Practice,
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
        Battle { core, fx: Fx::default(), cam: Camera::default(), mode, pending: Input::default(), acc: 0.0, debug: false }
    }

    pub fn practice(kit: usize) -> Self {
        Self::new(core::Battle::new([kit, PRACTICE_FOE_KIT], 0), Mode::Practice)
    }

    pub fn online(kit: usize, foe: Seat, endpoint: Endpoint) -> Self {
        let link = Link::open(&endpoint.url, &endpoint.ticket, &endpoint.cert_hash);
        let core = core::Battle::new([kit, foe.kit], endpoint.seat);
        Self::new(core, Mode::Online(Sync::new(link, endpoint.seat, foe.username)))
    }

    fn switch_kit(&mut self) {
        let Mode::Practice = self.mode else { return };
        if let Some(kit) = KIT_KEYS.iter().position(|k| is_key_pressed(*k)).filter(|k| *k != self.core.players[0].kit) {
            self.core = core::Battle::new([kit, PRACTICE_FOE_KIT], 0);
            self.fx = Fx::default();
        }
    }

    fn step(&mut self) {
        let foe_input = match &mut self.mode {
            Mode::Practice => Input::default(),
            Mode::Online(sync) => sync.remote_input(),
        };
        self.core.tick([self.pending, foe_input]);
        match &mut self.mode {
            Mode::Practice => {
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
            Mode::Practice => "practice   1/2/3 character   Esc menu".to_string(),
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
        let live = match &mut self.mode {
            Mode::Practice => true,
            Mode::Online(sync) => {
                if sync.finished(dt) {
                    return StackCommand::Pop;
                }
                sync.pull();
                sync.live()
            }
        };
        self.cam.follow(self.core.players[0].pos, self.fx.shake);
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
        view::draw_cursor();
        self.fx.draw_flash();
        view::draw_hud(&b.players);
        let tag = self.scoreline(ctx);
        let tw = measure_text(&tag, None, 20, 1.0).width;
        draw_text(&tag, (screen_width() - tw) / 2.0, 20.0, 20.0, LIGHTGRAY);
        if let Mode::Online(sync) = &self.mode {
            ui::text_centered(&sync.banner(), screen_height() / 2.0 - 120.0, 64, WHITE);
        }
        draw_text("WASD move   Mouse aim   RMB roll   Tab boxes", 40.0, screen_height() - 46.0, 24.0, LIGHTGRAY);
        draw_text(KITS[b.players[0].kit].hint, 40.0, screen_height() - 20.0, 24.0, LIGHTGRAY);
    }

    fn shows_cursor(&self) -> bool {
        false
    }
}
