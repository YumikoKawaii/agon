use macroquad::prelude::*;

use crate::{
    context::Context,
    core::kit::kit,
    net::{
        api::{Endpoint, Room, RoomReply},
        Pending, RoomWatch,
    },
    screen::{signed_out, Battle, Screen, StackCommand},
    ui::{self, Status},
};

const CARD: Vec2 = vec2(760.0, 500.0);
const SEAT: Vec2 = vec2(270.0, 274.0);
const SUBTLE: Color = Color::new(0.36, 0.32, 0.46, 1.0);

struct Layout {
    card: Rect,
    copy: Rect,
    seats: [Rect; 2],
    ready: Rect,
    leave: Rect,
}

fn layout() -> Layout {
    let card = Rect::new((screen_width() - CARD.x) / 2.0, 100.0 + ui::oy(), CARD.x, CARD.y);
    let sy = card.y + 112.0;
    Layout {
        card,
        copy: Rect::new(card.x + card.w - 40.0 - 150.0, card.y + 30.0, 150.0, 46.0),
        seats: [Rect::new(card.x + 40.0, sy, SEAT.x, SEAT.y), Rect::new(card.x + card.w - 40.0 - SEAT.x, sy, SEAT.x, SEAT.y)],
        ready: Rect::new(card.x + card.w - 40.0 - 200.0, card.y + card.h - 78.0, 200.0, 54.0),
        leave: Rect::new(card.x + card.w - 40.0 - 200.0 - 14.0 - 130.0, card.y + card.h - 74.0, 130.0, 46.0),
    }
}

fn dashes(x0: f32, x1: f32, y: f32) {
    let mut x = x0;
    while x < x1 {
        draw_rectangle(x, y, 8.0, 2.0, ui::with_alpha(ui::INK, 0.25));
        x += 14.0;
    }
}

fn pill_label(t: &str, cx: f32, y: f32, bg: Color, fg: Color) {
    let w = ui::chip_w(t, 18) + 12.0;
    let r = Rect::new(cx - w / 2.0, y, w, 28.0);
    ui::pill(r, bg);
    ui::text_in(t, r, r.y + 19.0, 18, fg);
}

pub struct Lobby {
    room: Room,
    seat: usize,
    watch: RoomWatch,
    ready: Option<Pending<RoomReply>>,
    endpoint: Option<Pending<Endpoint>>,
    joined_match: String,
    status: Status,
    lost: bool,
    retry: f32,
    probe: Option<Pending<RoomReply>>,
    copied: f32,
}

const RETRY_DELAY: f32 = 1.0;

impl Lobby {
    pub fn new(ctx: &Context, room: Room) -> Self {
        let seat = room.seat_of(&ctx.user.username);
        Lobby { watch: ctx.api.watch_room(&room.id), seat, room, ready: None, endpoint: None, joined_match: String::new(), status: Status::default(), lost: false, retry: 0.0, probe: None, copied: 0.0 }
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
    fn update(&mut self, ctx: &mut Context, dt: f32) -> StackCommand {
        ui::frame(80.0, 650.0);
        let (latest, closed) = if self.lost { (None, false) } else { self.watch.poll() };
        if let Some(room) = latest {
            self.apply(ctx, room);
        }
        if self.room.closed() {
            return StackCommand::Pop;
        }
        if closed {
            self.lost = true;
            self.retry = RETRY_DELAY;
        }
        if self.lost && self.probe.is_none() {
            self.retry -= dt;
            if self.retry <= 0.0 {
                self.probe = Some(ctx.api.get_room(&self.room.id));
            }
        }
        if let Some(result) = self.probe.as_mut().and_then(Pending::poll) {
            self.probe = None;
            match result {
                Ok(r) => {
                    self.status.error.clear();
                    self.apply(ctx, r.room);
                    self.watch = ctx.api.watch_room(&self.room.id);
                    self.lost = false;
                }
                Err(e) if e.unauthenticated => return signed_out(ctx),
                Err(e) => {
                    self.status.error = e.message;
                    self.retry = RETRY_DELAY;
                }
            }
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
        let l = layout();
        if ui::clicked(l.ready, idle && self.room.waiting()) {
            self.status.error.clear();
            self.ready = Some(ctx.api.set_ready(&self.room.id, !self.me()));
        }
        if ui::clicked(l.copy, true) {
            macroquad::miniquad::window::clipboard_set(&self.room.id);
            self.copied = 1.5;
        }
        self.copied = (self.copied - dt).max(0.0);
        if ui::clicked(l.leave, idle) || (idle && is_key_pressed(KeyCode::Escape)) {
            drop(ctx.api.leave_room(&self.room.id));
            return StackCommand::Pop;
        }
        StackCommand::None
    }

    fn draw(&self, _ctx: &Context) {
        let l = layout();
        let c = l.card;
        ui::panel_scaled(c, 0.55);
        draw_text("ROOM", c.x + 42.0, c.y + 62.0, 20.0, SUBTLE);
        draw_text(&self.room.id, c.x + 108.0, c.y + 66.0, 44.0, ui::INK);
        ui::button(l.copy, if self.copied > 0.0 { "Copied" } else { "Copy id" }, true);
        dashes(c.x + 40.0, c.x + c.w - 40.0, c.y + 96.0);

        let kit_ui = &crate::assets::get().ui;
        for (i, r) in l.seats.iter().enumerate() {
            let seat = self.room.seat(i);
            let empty = seat.username.is_empty();
            let mine = i == self.seat;
            draw_rectangle(r.x, r.y, r.w, r.h, ui::with_alpha(WHITE, if mine { 0.5 } else { 0.32 }));
            if seat.ready && !empty {
                draw_rectangle_lines(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 5.0, ui::MINT_D);
            } else {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, ui::with_alpha(ui::INK, 0.2));
            }
            ui::text_in(if i == 0 { "HOST" } else { "GUEST" }, *r, r.y + 30.0, 16, SUBTLE);
            let pr = Rect::new(r.x + (r.w - 110.0) / 2.0, r.y + 44.0, 110.0, 110.0);
            if empty {
                let a = 0.75 + 0.25 * (get_time() as f32 * 3.0).sin();
                draw_rectangle(pr.x - 3.0, pr.y - 3.0, pr.w + 6.0, pr.h + 6.0, ui::INK);
                draw_rectangle(pr.x, pr.y, pr.w, pr.h, Color::new(0.38, 0.33, 0.55, 1.0));
                let t = &kit_ui.unknown;
                draw_texture_ex(t, pr.x, pr.y, Color::new(1.0, 1.0, 1.0, a), DrawTextureParams { dest_size: Some(pr.size()), ..Default::default() });
                ui::text_in("Empty seat", *r, r.y + 194.0, 26, SUBTLE);
                pill_label("Waiting for opponent", r.x + r.w / 2.0, r.y + 222.0, ui::with_alpha(ui::INK, 0.1), SUBTLE);
                continue;
            }
            ui::portrait_box(seat.kit, pr, if i == 0 { ui::MINT } else { ui::ROSE }, i == 1);
            ui::text_in(&seat.username, *r, r.y + 194.0, 30, ui::INK);
            let k = kit(seat.kit);
            ui::text_in(&format!("{}  -  {}", k.name, k.role.name()), *r, r.y + 216.0, 18, SUBTLE);
            if seat.ready {
                pill_label("Ready", r.x + r.w / 2.0, r.y + 230.0, ui::MINT_D, WHITE);
            } else {
                pill_label("Not ready", r.x + r.w / 2.0, r.y + 230.0, ui::with_alpha(ui::INK, 0.1), SUBTLE);
            }
        }
        let vs = &kit_ui.versus;
        let vw = 120.0;
        let vh = vw * vs.height() / vs.width();
        let vc = vec2(c.x + c.w / 2.0, l.seats[0].y + SEAT.y / 2.0);
        draw_texture_ex(vs, (vc.x - vw / 2.0).round(), (vc.y - vh / 2.0).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(vw, vh)), ..Default::default() });

        dashes(c.x + 40.0, c.x + c.w - 40.0, c.y + c.h - 92.0);
        draw_text("Share the room id with your opponent", c.x + 42.0, c.y + c.h - 44.0, 18.0, SUBTLE);
        let idle = self.ready.is_none() && self.endpoint.is_none();
        let can = idle && self.room.waiting();
        ui::plain(l.leave, "Leave", idle);
        if self.me() {
            ui::button(l.ready, "Unready", can);
        } else {
            ui::mint_button(l.ready, "Ready", can);
        }
        self.status.draw();
    }
}
