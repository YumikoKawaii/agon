use macroquad::prelude::*;

use crate::{
    assets,
    context::Context,
    core::{anim::State, KITS},
    net::{api::UserReply, Pending},
    screen::{signed_out, Screen, StackCommand},
    ui::{self, Status},
};

const CARD_W: f32 = 260.0;
const CARD_GAP: f32 = 30.0;
const SOUTH: usize = 2;

fn cards() -> Vec<Rect> {
    let x0 = (screen_width() - (CARD_W * KITS.len() as f32 + CARD_GAP * (KITS.len() - 1) as f32)) / 2.0;
    (0..KITS.len()).map(|i| Rect::new(x0 + i as f32 * (CARD_W + CARD_GAP), 160.0, CARD_W, 380.0)).collect()
}

fn confirm_rect() -> Rect {
    ui::centered(320.0, 580.0, 56.0)
}

pub fn draw_idle(kit: usize, x: f32, y: f32, w: f32, scale: f32) {
    let tex = assets::skin(kit).frame(State::Idle, SOUTH, (get_time() * 60.0) as u32, 1);
    let size = vec2(tex.width(), tex.height()) * scale;
    draw_texture_ex(tex, x + (w - size.x) / 2.0, y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
}

pub struct Characters {
    pick: usize,
    pending: Option<Pending<UserReply>>,
    status: Status,
}

impl Characters {
    pub fn new(current: usize) -> Self {
        Characters { pick: current, pending: None, status: Status::default() }
    }
}

impl Screen for Characters {
    fn update(&mut self, ctx: &mut Context, _dt: f32) -> StackCommand {
        if let Some(p) = self.pending.as_mut() {
            if let Some(result) = p.poll() {
                self.pending = None;
                match result {
                    Ok(r) => {
                        ctx.user.kit = r.user.kit;
                        return StackCommand::Pop;
                    }
                    Err(e) if e.unauthenticated => return signed_out(ctx),
                    Err(e) => self.status.error = e.message,
                }
            }
        }
        let idle = self.pending.is_none();
        self.status.busy = !idle;
        if let Some(i) = cards().iter().position(|r| ui::clicked(*r, idle)) {
            self.pick = i;
        }
        if ui::clicked(confirm_rect(), idle) {
            self.pending = Some(ctx.api.set_kit(self.pick));
        }
        if idle && is_key_pressed(KeyCode::Escape) {
            return StackCommand::Pop;
        }
        StackCommand::None
    }

    fn draw(&self, _ctx: &Context) {
        ui::text_centered("Choose your character", 110.0, ui::HEADING, WHITE);
        for (i, r) in cards().into_iter().enumerate() {
            ui::panel(r, self.pick == i);
            draw_idle(i, r.x, r.y + 20.0, r.w, 3.0);
            ui::text_in(KITS[i].name, r, r.y + 250.0, 28, WHITE);
            ui::text_in(&format!("HP {}", KITS[i].hp), r, r.y + 285.0, ui::SMALL, ui::DIM);
        }
        ui::button(confirm_rect(), "Confirm", self.pending.is_none());
        self.status.draw();
    }
}
