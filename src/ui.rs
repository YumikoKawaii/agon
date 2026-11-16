use macroquad::prelude::*;

pub const BG: Color = Color::new(0.13, 0.11, 0.19, 1.0);
pub const ACCENT: Color = Color::new(0.45, 1.0, 0.92, 1.0);
pub const DIM: Color = Color::new(0.6, 0.63, 0.72, 1.0);
pub const ERROR: Color = Color::new(1.0, 0.5, 0.5, 1.0);

pub const TITLE: u16 = 96;
pub const HEADING: u16 = 48;
pub const BODY: u16 = 26;
pub const SMALL: u16 = 22;

pub fn hovered(r: Rect) -> bool {
    r.contains(Vec2::from(mouse_position()))
}

pub fn clicked(r: Rect, enabled: bool) -> bool {
    enabled && hovered(r) && is_mouse_button_pressed(MouseButton::Left)
}

thread_local! {
    static FRAME: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

pub fn frame(top: f32, bottom: f32) {
    FRAME.with(|f| f.set((top, bottom)));
}

pub fn oy() -> f32 {
    let (top, bottom) = FRAME.with(|f| f.get());
    ((screen_height() - (bottom - top)) / 2.0 - top).max(0.0)
}

pub fn centered(w: f32, y: f32, h: f32) -> Rect {
    Rect::new((screen_width() - w) / 2.0, y + oy(), w, h)
}

pub fn centered_abs(w: f32, y: f32, h: f32) -> Rect {
    Rect::new((screen_width() - w) / 2.0, y, w, h)
}

pub fn row(r: Rect, n: usize, gap: f32) -> Vec<Rect> {
    let w = (r.w - gap * (n - 1) as f32) / n as f32;
    (0..n).map(|i| Rect::new(r.x + i as f32 * (w + gap), r.y, w, r.h)).collect()
}

pub fn text_centered(t: &str, y: f32, size: u16, c: Color) {
    text_centered_abs(t, y + oy(), size, c);
}

pub fn text_centered_abs(t: &str, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, (screen_width() - w) / 2.0, y, size as f32, c);
}

pub fn text_in(t: &str, r: Rect, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, r.x + (r.w - w) / 2.0, y, size as f32, c);
}

pub const INK: Color = Color::new(0.16, 0.14, 0.25, 1.0);
pub const GOLD: Color = Color::new(0.95, 0.76, 0.38, 1.0);

pub fn form(top: f32, h: f32) {
    panel_scaled(centered(440.0, top, h), 0.55);
}

fn kit() -> &'static crate::assets::UiKit {
    &crate::assets::get().ui
}

pub fn logo(y: f32, scale: f32) {
    let t = &kit().logo;
    let size = vec2(t.width(), t.height()) * scale;
    draw_texture_ex(t, (screen_width() - size.x) / 2.0, y + oy() - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
}

pub fn reticle(at: Vec2, scale: f32) {
    let t = &kit().reticle;
    let size = vec2(t.width(), t.height()) * scale;
    draw_texture_ex(t, at.x - size.x / 2.0, at.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
}

pub fn bar(r: Rect, frac: f32, fill: Color, right: bool) {
    kit().bar.draw(r, r.h / 22.0, WHITE);
    let (px, py) = (r.h * 0.5, r.h * 0.3);
    let inner = r.w - 2.0 * px;
    let w = inner * frac.clamp(0.0, 1.0);
    let x = if right { r.x + r.w - px - w } else { r.x + px };
    draw_rectangle(r.x + px, r.y + py, inner, r.h - 2.0 * py, Color::new(0.23, 0.2, 0.31, 1.0));
    draw_rectangle(x, r.y + py, w, r.h - 2.0 * py, fill);
    draw_rectangle(x, r.y + py, w, (r.h - 2.0 * py) * 0.35, Color::new(1.0, 1.0, 1.0, 0.3));
}

pub fn panel_scaled(r: Rect, scale: f32) {
    kit().panel.draw(r, scale, WHITE);
}

pub fn frame_border(r: Rect) {
    let c = INK;
    draw_rectangle_lines(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, 4.0, Color::new(0.5, 0.78, 0.7, 1.0));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, c);
    draw_rectangle_lines(r.x + 5.0, r.y + 5.0, r.w - 10.0, r.h - 10.0, 1.5, c);
}

pub fn slot(r: Rect, tint: Color) {
    kit().button.draw(r, r.h / 68.0 * 1.2, tint);
}

pub fn panel(r: Rect, highlight: bool) {
    let tint = if highlight { Color::new(1.0, 0.95, 0.8, 1.0) } else if hovered(r) { Color::new(1.0, 1.0, 0.95, 1.0) } else { Color::new(0.92, 0.92, 0.92, 1.0) };
    let r = if highlight { Rect::new(r.x - 6.0, r.y - 10.0, r.w + 12.0, r.h + 12.0) } else { r };
    kit().panel.draw(r, 1.0, tint);
    if highlight {
        draw_rectangle(r.x + 30.0, r.y + r.h - 34.0, r.w - 60.0, 4.0, GOLD);
    }
}

pub fn button(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    let tint = if !enabled { Color::new(0.55, 0.55, 0.6, 1.0) } else if h { Color::new(1.1, 1.1, 1.1, 1.0) } else { WHITE };
    let y = if h && is_mouse_button_down(MouseButton::Left) { 2.0 } else { 0.0 };
    kit().button.draw(Rect::new(r.x, r.y + y, r.w, r.h), r.h / 68.0 * 1.6, tint);
    text_in(label, r, r.y + y + r.h / 2.0 + 7.0, BODY, if enabled { WHITE } else { DIM });
}

pub struct TextField {
    pub value: String,
    max: usize,
    secret: bool,
}

impl TextField {
    pub fn name(max: usize) -> Self {
        TextField { value: String::new(), max, secret: false }
    }

    pub fn secret(max: usize) -> Self {
        TextField { value: String::new(), max, secret: true }
    }

    pub fn input(&mut self) {
        while let Some(c) = get_char_pressed() {
            let ok = if self.secret { c.is_ascii_graphic() } else { c.is_ascii_alphanumeric() || c == '_' || c == '-' };
            if ok && self.value.len() < self.max {
                self.value.push(c);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.value.pop();
        }
    }

    pub fn draw(&self, r: Rect, label: &str, focused: bool) {
        draw_text(label, r.x + 2.0, r.y - 8.0, 20.0, INK);
        kit().field.draw(r, 1.5, if focused { WHITE } else { Color::new(0.88, 0.88, 0.9, 1.0) });
        let shown = if self.secret { "*".repeat(self.value.len()) } else { self.value.clone() };
        let caret = if focused && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
        draw_text(&format!("{shown}{caret}"), r.x + 16.0, r.y + r.h / 2.0 + 9.0, BODY as f32, INK);
    }
}

#[derive(Default)]
pub struct Status {
    pub error: String,
    pub busy: bool,
}

impl Status {
    pub fn draw(&self) {
        text_centered_abs(&self.error, screen_height() - 70.0, SMALL, ERROR);
        if self.busy {
            text_centered_abs("...", screen_height() - 100.0, 28, DIM);
        }
    }
}
