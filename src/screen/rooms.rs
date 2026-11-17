use macroquad::prelude::*;

use crate::{
    context::Context,
    core::kit::kit,
    net::{api::RoomReply, Pending},
    screen::{signed_out, Lobby, Screen, StackCommand},
    ui::{self, Status, TextField},
};

const ROOM_ID_MAX: usize = 12;

struct Layout {
    create: Rect,
    room_id: Rect,
    join: Rect,
    back: Rect,
}

const CARD: Vec2 = vec2(380.0, 300.0);
const GAP: f32 = 40.0;

fn cards() -> [Rect; 2] {
    let x = (screen_width() - CARD.x * 2.0 - GAP) / 2.0;
    let y = 210.0 + ui::oy();
    [Rect::new(x, y, CARD.x, CARD.y), Rect::new(x + CARD.x + GAP, y, CARD.x, CARD.y)]
}

fn layout() -> Layout {
    let [a, b] = cards();
    Layout {
        create: Rect::new(a.x + 40.0, a.y + a.h - 100.0, a.w - 80.0, 60.0),
        room_id: Rect::new(b.x + 40.0, b.y + 130.0, b.w - 80.0, 48.0),
        join: Rect::new(b.x + 40.0, b.y + b.h - 100.0, b.w - 80.0, 60.0),
        back: ui::centered(160.0, 550.0, 46.0),
    }
}

pub struct Rooms {
    room_id: TextField,
    pending: Option<Pending<RoomReply>>,
    status: Status,
}

impl Rooms {
    pub fn new() -> Self {
        Rooms { room_id: TextField::name(ROOM_ID_MAX), pending: None, status: Status::default() }
    }
}

impl Screen for Rooms {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        ui::frame(110.0, 600.0);
        if let Some(p) = self.pending.as_mut() {
            if let Some(result) = p.poll() {
                self.pending = None;
                match result {
                    Ok(r) => return StackCommand::Push(Box::new(Lobby::new(ctx, r.room))),
                    Err(e) if e.unauthenticated => return signed_out(ctx),
                    Err(e) => self.status.error = e.message,
                }
            }
        }
        let l = layout();
        let idle = self.pending.is_none();
        self.status.busy = !idle;
        self.room_id.input();
        if ui::clicked(l.create, idle) {
            self.status.error.clear();
            self.pending = Some(ctx.api.create_room());
        }
        let join = ui::clicked(l.join, idle) || (idle && is_key_pressed(KeyCode::Enter));
        if join && !self.room_id.value.is_empty() {
            self.status.error.clear();
            self.pending = Some(ctx.api.join_room(&self.room_id.value));
        }
        if ui::clicked(l.back, idle) || (idle && is_key_pressed(KeyCode::Escape)) {
            return StackCommand::Pop;
        }
        StackCommand::None
    }

    fn draw(&self, ctx: &Context) {
        ui::frame(110.0, 600.0);
        let l = layout();
        let idle = self.pending.is_none();
        let head = Rect::new(0.0, 0.0, screen_width(), 0.0);
        ui::text_out_in("Play online", head, 150.0 + ui::oy(), 64, ui::GOLD, 3.0);
        ui::text_in(&format!("playing as {}", kit(ctx.user.kit).name), head, 186.0 + ui::oy(), ui::SMALL + 2, ui::PALE);
        let [a, b] = cards();
        ui::panel_scaled(a, 0.55);
        ui::panel_scaled(b, 0.55);
        ui::text_in("Host", a, a.y + 62.0, 36, ui::INK);
        for (i, line) in ["Create a room and share", "its id with your opponent."].iter().enumerate() {
            ui::text_in(line, a, a.y + 108.0 + i as f32 * 24.0, 20, Color::new(0.36, 0.32, 0.46, 1.0));
        }
        ui::text_in("Join", b, b.y + 62.0, 36, ui::INK);
        ui::mint_button(l.create, "Create room", idle);
        self.room_id.draw(l.room_id, "room id", true);
        ui::button(l.join, "Join", idle && !self.room_id.value.is_empty());
        ui::ghost(l.back, "Back", idle);
        self.status.draw();
    }
}
