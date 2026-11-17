use macroquad::prelude::*;

use crate::{
    assets,
    context::Context,
    core::{anim::State, KITS},
    net::{api::UserReply, Pending},
    screen::{signed_out, Screen, StackCommand},
    ui::{self, Status},
};

const CARD_W: f32 = 220.0;
const CARD_GAP: f32 = 20.0;
const CARD_H: f32 = 380.0;
const CARD_MARGIN: f32 = 24.0;
const SOUTH: usize = 2;

fn card_scale() -> f32 {
    let n = KITS.len() as f32;
    ((screen_width() - CARD_MARGIN * 2.0) / (CARD_W * n + CARD_GAP * (n - 1.0))).min(1.0)
}

fn cards() -> Vec<Rect> {
    let k = card_scale();
    let (w, gap) = (CARD_W * k, CARD_GAP * k);
    let x0 = (screen_width() - (w * KITS.len() as f32 + gap * (KITS.len() - 1) as f32)) / 2.0;
    (0..KITS.len()).map(|i| Rect::new(x0 + i as f32 * (w + gap), 160.0 + ui::oy(), w, CARD_H * k)).collect()
}

fn confirm_rect() -> Rect {
    ui::centered(320.0, 650.0, 56.0)
}

const IDLE_BOX: f32 = 88.0;

pub fn draw_idle(kit: usize, x: f32, y: f32, w: f32, scale: f32) {
    let tex = assets::skin(kit).frame(State::Idle, SOUTH, (get_time() * 60.0) as u32, 1);
    let size = vec2(tex.width(), tex.height()) * scale;
    let centre = vec2(x + w / 2.0, y + IDLE_BOX * scale / 2.0);
    draw_texture_ex(tex, centre.x - size.x / 2.0, centre.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
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
        ui::frame(80.0, 720.0);
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
        ui::frame(80.0, 720.0);
        ui::text_centered("Choose your character", 110.0, ui::HEADING, ui::GOLD);
        for (i, part) in KITS[self.pick].hint.split("   ").enumerate() {
            ui::text_centered(part, 562.0 + i as f32 * 20.0, ui::SMALL - 4, ui::DIM);
        }
        let k = card_scale();
        let px = |v: f32| (v * k).round() as u16;
        for (i, r) in cards().into_iter().enumerate() {
            ui::panel(r, self.pick == i);
            draw_idle(i, r.x, r.y + 30.0 * k, r.w, 2.6 * k);
            ui::text_in(KITS[i].name, r, r.y + 282.0 * k, px(26.0), ui::INK);
            ui::text_in(&format!("HP {}", KITS[i].hp), r, r.y + 308.0 * k, px(ui::SMALL as f32), ui::INK);
            let (s, gap, pad) = (34.0 * k, 8.0 * k, 4.0 * k);
            let x0 = r.x + (r.w - (4.0 * s + 3.0 * gap)) / 2.0;
            for (n, skill) in KITS[i].skills.iter().enumerate() {
                let ir = Rect::new(x0 + n as f32 * (s + gap), r.y + 320.0 * k, s, s);
                ui::slot(ir, WHITE);
                if let Some(t) = assets::skill_icon(*skill) {
                    draw_texture_ex(t, ir.x + pad, ir.y + pad, WHITE, DrawTextureParams { dest_size: Some(vec2(s - 2.0 * pad, s - 2.0 * pad)), ..Default::default() });
                }
            }
        }
        ui::button(confirm_rect(), "Confirm", self.pending.is_none());
        self.status.draw();
    }
}
