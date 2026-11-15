use macroquad::prelude::*;

pub const BG: Color = Color::new(0.05, 0.06, 0.09, 1.0);
pub const PANEL: Color = Color::new(0.13, 0.15, 0.2, 1.0);
pub const EDGE: Color = Color::new(0.35, 0.38, 0.5, 1.0);
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

pub fn centered(w: f32, y: f32, h: f32) -> Rect {
    Rect::new((screen_width() - w) / 2.0, y, w, h)
}

pub fn row(r: Rect, n: usize, gap: f32) -> Vec<Rect> {
    let w = (r.w - gap * (n - 1) as f32) / n as f32;
    (0..n).map(|i| Rect::new(r.x + i as f32 * (w + gap), r.y, w, r.h)).collect()
}

pub fn text_centered(t: &str, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, (screen_width() - w) / 2.0, y, size as f32, c);
}

pub fn text_in(t: &str, r: Rect, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, r.x + (r.w - w) / 2.0, y, size as f32, c);
}

pub fn panel(r: Rect, highlight: bool) {
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, if highlight { 4.0 } else { 2.0 }, if highlight || hovered(r) { ACCENT } else { EDGE });
}

pub fn button(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    draw_rectangle(r.x, r.y, r.w, r.h, if h { EDGE } else { PANEL });
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if h { ACCENT } else { EDGE });
    text_in(label, r, r.y + r.h / 2.0 + 9.0, BODY, if enabled { WHITE } else { DIM });
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
        draw_text(label, r.x, r.y - 8.0, 20.0, DIM);
        draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if focused { ACCENT } else { EDGE });
        let shown = if self.secret { "*".repeat(self.value.len()) } else { self.value.clone() };
        let caret = if focused && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
        draw_text(&format!("{shown}{caret}"), r.x + 12.0, r.y + r.h / 2.0 + 9.0, BODY as f32, WHITE);
    }
}

#[derive(Default)]
pub struct Status {
    pub error: String,
    pub busy: bool,
}

impl Status {
    pub fn draw(&self) {
        text_centered(&self.error, screen_height() - 70.0, SMALL, ERROR);
        if self.busy {
            text_centered("...", screen_height() - 100.0, 28, DIM);
        }
    }
}
