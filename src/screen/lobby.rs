use macroquad::prelude::*;

use crate::{
    context::Context,
    core::kit::kit,
    net::{
        api::{Endpoint, Room, RoomReply},
        Pending, RoomWatch,
    },
    screen::{characters::draw_idle, signed_out, Battle, Screen, StackCommand},
    ui::{self, Status},
};

const SEAT_W: f32 = 300.0;
const SEAT_GAP: f32 = 140.0;

fn seats() -> [Rect; 2] {
    let x0 = (screen_width() - (SEAT_W * 2.0 + SEAT_GAP)) / 2.0;
    [Rect::new(x0, 180.0 + ui::oy(), SEAT_W, 300.0), Rect::new(x0 + SEAT_W + SEAT_GAP, 180.0 + ui::oy(), SEAT_W, 300.0)]
}

fn ready_rect() -> Rect {
    ui::centered(320.0, 510.0, 56.0)
}

fn leave_rect() -> Rect {
    ui::centered(320.0, 580.0, 52.0)
}

pub struct Lobby {
    room: Room,
    seat: usize,
    watch: RoomWatch,
    ready: Option<Pending<RoomReply>>,
    endpoint: Option<Pending<Endpoint>>,
    joined_match: String,
    status: Status,
}

impl Lobby {
    pub fn new(ctx: &Context, room: Room) -> Self {
        let seat = room.seat_of(&ctx.user.username);
        Lobby { watch: ctx.api.watch_room(&room.id), seat, room, ready: None, endpoint: None, joined_match: String::new(), status: Status::default() }
    }

    fn me(&self) -> bool {
        self.room.seat(self.seat).ready
    }

    fn apply(&mut self, ctx: &Context, room: Room) {
        if room.playing() && !room.match_id.is_empty() && room.match_id != self.joined_match && self.endpoint.is_none() {
            self.joined_match = room.match_id.clone();
            self.endpoint = Some(ctx.api.endpoint(&room.match_id));
        }
        self.room = room;
    }
}

impl Screen for Lobby {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        ui::frame(80.0, 650.0);
        let (latest, closed) = self.watch.poll();
        if let Some(room) = latest {
            self.apply(ctx, room);
        }
        if self.room.closed() {
            return StackCommand::Pop;
        }
        if closed {
            self.watch = ctx.api.watch_room(&self.room.id);
        }
        if let Some(result) = self.ready.as_mut().and_then(Pending::poll) {
            self.ready = None;
            match result {
                Ok(r) => self.apply(ctx, r.room),
                Err(e) if e.unauthenticated => return signed_out(ctx),
                Err(e) => self.status.error = e.message,
            }
        }
        if let Some(result) = self.endpoint.as_mut().and_then(Pending::poll) {
            self.endpoint = None;
            match result {
                Ok(e) => {
                    let foe = self.room.seat(1 - e.seat).clone();
                    return StackCommand::Push(Box::new(Battle::online(ctx.user.kit, foe, e)));
                }
                Err(e) if e.unauthenticated => return signed_out(ctx),
                Err(e) => self.status.error = e.message,
            }
        }
        let idle = self.ready.is_none() && self.endpoint.is_none();
        self.status.busy = !idle;
        if ui::clicked(ready_rect(), idle && self.room.waiting()) {
            self.status.error.clear();
            self.ready = Some(ctx.api.set_ready(&self.room.id, !self.me()));
        }
        if ui::clicked(leave_rect(), idle) || (idle && is_key_pressed(KeyCode::Escape)) {
            drop(ctx.api.leave_room(&self.room.id));
            return StackCommand::Pop;
        }
        StackCommand::None
    }

    fn draw(&self, _ctx: &Context) {
        ui::frame(80.0, 650.0);
        ui::text_centered(&format!("Room {}", self.room.id), 110.0, 56, ui::GOLD);
        ui::text_centered("VS", 350.0, 64, ui::GOLD);
        ui::text_centered("share this id with your opponent", 145.0, ui::SMALL, ui::DIM);
        for (i, r) in seats().into_iter().enumerate() {
            let seat = self.room.seat(i);
            ui::panel(r, seat.ready);
            ui::text_in(if i == 0 { "HOST" } else { "GUEST" }, r, r.y + 54.0, 20, ui::INK);
            if seat.username.is_empty() {
                ui::text_in("waiting...", r, r.y + 160.0, ui::BODY, ui::INK);
                continue;
            }
            draw_idle(seat.kit, r.x, r.y + 60.0, r.w, 2.2);
            ui::text_in(&format!("{}  ({})", seat.username, kit(seat.kit).name), r, r.y + 232.0, ui::SMALL, ui::INK);
            let (label, color) = if seat.ready { ("READY", Color::new(0.2, 0.55, 0.45, 1.0)) } else { ("not ready", ui::INK) };
            ui::text_in(label, r, r.y + 262.0, 24, color);
        }
        let idle = self.ready.is_none() && self.endpoint.is_none();
        ui::button(ready_rect(), if self.me() { "Unready" } else { "Ready" }, idle && self.room.waiting());
        ui::button(leave_rect(), "Leave", idle);
        self.status.draw();
    }
}
