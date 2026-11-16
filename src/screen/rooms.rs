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

fn layout() -> Layout {
    Layout {
        create: ui::centered(360.0, 222.0, 56.0),
        room_id: ui::centered(360.0, 360.0, 48.0),
        join: ui::centered(360.0, 428.0, 52.0),
        back: ui::centered(360.0, 530.0, 52.0),
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
        let l = layout();
        let idle = self.pending.is_none();
        ui::text_centered("Play online", 140.0, 56, ui::GOLD);
        ui::text_centered(&format!("playing as {}", kit(ctx.user.kit).name), 180.0, ui::SMALL + 2, ui::DIM);
        ui::form(196.0, 310.0);
        ui::button(l.create, "Create room", idle);
        ui::text_centered("or join by id", 315.0, ui::SMALL, ui::INK);
        self.room_id.draw(l.room_id, "room id", true);
        ui::button(l.join, "Join", idle);
        ui::button(l.back, "Back", idle);
        self.status.draw();
    }
}
