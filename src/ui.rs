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
    static FONTS: std::cell::OnceCell<(Font, Font)> = const { std::cell::OnceCell::new() };
}

pub async fn load_fonts() {
    let (Ok(mut title), Ok(mut body)) = (load_ttf_font("assets/ui/font_title.ttf").await, load_ttf_font("assets/ui/font_body.ttf").await) else {
        return;
    };
    title.set_filter(FilterMode::Nearest);
    body.set_filter(FilterMode::Nearest);
    FONTS.with(|f| {
        let _ = f.set((title, body));
    });
}

fn font_for(size: f32) -> Option<Font> {
    FONTS.with(|f| f.get().map(|(t, b)| if size >= 40.0 { t.clone() } else { b.clone() }))
}

pub fn draw_text(t: &str, x: f32, y: f32, size: f32, c: Color) {
    let c = if c == INK { PALE } else { c };
    draw_text_raw(t, x, y, size, c);
}

pub mod raw {
    pub use super::{draw_text_raw as draw_text, measure_text};
}

pub fn hud_panel(r: Rect, scale: f32) {
    kit().hud.draw(r, scale, WHITE);
}

pub fn draw_text_raw(t: &str, x: f32, y: f32, size: f32, c: Color) {
    let font = font_for(size);
    draw_text_ex(t, x.round(), y.round(), TextParams { font: font.as_ref(), font_size: size as u16, color: c, ..Default::default() });
}

pub fn measure_text(t: &str, _font: Option<&Font>, size: u16, scale: f32) -> TextDimensions {
    let font = font_for(size as f32);
    macroquad::text::measure_text(t, font.as_ref(), size, scale)
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

pub fn label_in(t: &str, r: Rect, size: u16, c: Color) {
    let max_h = ((r.h * 0.42) as u16).max(12);
    let mut size = size.min(max_h);
    while size > 12 && measure_text(t, None, size, 1.0).width > r.w - 24.0 {
        size -= 1;
    }
    let d = measure_text("Ag", None, size, 1.0);
    let w = measure_text(t, None, size, 1.0).width;
    let base = r.y + (r.h - d.height) / 2.0 + d.offset_y;
    draw_text_raw(t, r.x + (r.w - w) / 2.0, base + 1.0, size as f32, SHADOW);
    draw_text(t, r.x + (r.w - w) / 2.0, base - 1.0, size as f32, c);
}

pub fn flat_panel(r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.1, 0.08, 0.19, 0.9));
    draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, 2.0, with_alpha(WHITE, 0.08));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, with_alpha(MINT, 0.55));
    for (x, y) in [(r.x, r.y), (r.x + r.w - 6.0, r.y), (r.x, r.y + r.h - 6.0), (r.x + r.w - 6.0, r.y + r.h - 6.0)] {
        draw_rectangle(x, y, 6.0, 6.0, GOLD);
    }
}

pub fn text_in(t: &str, r: Rect, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, r.x + (r.w - w) / 2.0, y, size as f32, c);
}

pub const INK: Color = Color::new(0.16, 0.14, 0.25, 1.0);
pub const GOLD: Color = Color::new(0.95, 0.76, 0.38, 1.0);
pub const MINT: Color = Color::new(0.56, 0.86, 0.79, 1.0);
pub const MINT_D: Color = Color::new(0.31, 0.66, 0.58, 1.0);
pub const LAV_D: Color = Color::new(0.48, 0.39, 0.77, 1.0);
pub const ROSE: Color = Color::new(0.93, 0.58, 0.67, 1.0);
pub const ROSE_D: Color = Color::new(0.82, 0.42, 0.53, 1.0);
pub const CREAM: Color = Color::new(0.96, 0.91, 0.79, 1.0);
pub const PALE: Color = Color::new(0.91, 0.88, 0.98, 1.0);
const NIGHT: [Color; 2] = [Color::new(0.23, 0.18, 0.39, 1.0), Color::new(0.16, 0.29, 0.34, 1.0)];

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a.clamp(0.0, 1.0))
}

pub fn backdrop() {
    backdrop_in(screen_width(), screen_height());
}

pub fn backdrop_in(w: f32, h: f32) {
    let bands = 48;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let c = Color::new(NIGHT[0].r + (NIGHT[1].r - NIGHT[0].r) * t, NIGHT[0].g + (NIGHT[1].g - NIGHT[0].g) * t, NIGHT[0].b + (NIGHT[1].b - NIGHT[0].b) * t, 1.0);
        let y = h * i as f32 / bands as f32;
        draw_rectangle(0.0, y, w, h / bands as f32 + 1.0, c);
    }
    for (x, y, r, c, a) in [(0.25, 1.05, 0.55, MINT, 0.10), (0.92, -0.05, 0.45, ROSE, 0.08)] {
        for k in 0..6 {
            let f = 1.0 - k as f32 / 6.0;
            draw_circle(w * x, h * y, w * r * f, with_alpha(c, a * 0.35));
        }
    }
    let t = get_time() as f32;
    for i in 0..70u32 {
        let hx = (i.wrapping_mul(2_654_435_761) >> 8) as f32 / 16_777_216.0;
        let hy = (i.wrapping_mul(2_246_822_519).wrapping_add(97) >> 8) as f32 / 16_777_216.0;
        let tw = 0.5 + 0.5 * (t * (0.6 + hx * 1.4) + i as f32).sin();
        let s = if i % 7 == 0 { 2.5 } else { 1.5 };
        let y = (hy * h - t * (4.0 + hx * 6.0)).rem_euclid(h);
        draw_rectangle((hx * w).round(), y.round(), s, s, with_alpha(WHITE, 0.25 + 0.6 * tw));
    }
}

pub fn pedestal(centre: Vec2, w: f32) {
    for k in 0..5 {
        let f = 1.0 - k as f32 / 5.0;
        draw_ellipse(centre.x, centre.y - w * 0.05, w * 0.75 * f, w * 0.28 * f, 0.0, with_alpha(MINT, 0.07));
    }
    draw_ellipse(centre.x, centre.y + w * 0.035, w * 0.5, w * 0.12, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));
    draw_ellipse(centre.x, centre.y, w * 0.5, w * 0.12, 0.0, MINT_D);
    draw_ellipse(centre.x, centre.y - w * 0.02, w * 0.38, w * 0.07, 0.0, with_alpha(WHITE, 0.18));
    draw_ellipse_lines(centre.x, centre.y, w * 0.5, w * 0.12, 0.0, 3.0, INK);
}

pub fn pill(r: Rect, c: Color) {
    let rad = r.h / 2.0;
    draw_rectangle(r.x + rad, r.y, r.w - 2.0 * rad, r.h, c);
    draw_circle(r.x + rad, r.y + rad, rad, c);
    draw_circle(r.x + r.w - rad, r.y + rad, rad, c);
}

pub fn chip_w(t: &str, size: u16) -> f32 {
    measure_text(t, None, size, 1.0).width + size as f32 * 1.2
}

pub fn chip(t: &str, x: f32, y: f32, size: u16, bg: Color, fg: Color) -> Rect {
    let r = Rect::new(x, y, chip_w(t, size), size as f32 * 1.35);
    pill(r, bg);
    text_in(t, r, r.y + r.h * 0.5 + size as f32 * 0.3, size, fg);
    r
}

pub fn keycap(t: &str, cx: f32, y: f32, size: u16) {
    let w = measure_text(t, None, size, 1.0).width + 10.0;
    let r = Rect::new(cx - w / 2.0, y, w, size as f32 + 4.0);
    draw_rectangle(r.x, r.y, r.w, r.h, INK);
    text_in(t, r, r.y + r.h * 0.5 + size as f32 * 0.32, size, CREAM);
}

pub fn text_out(t: &str, x: f32, y: f32, size: u16, c: Color, out: f32) {
    draw_text_raw(t, x, y + (out * 2.0).max(2.0), size as f32, SHADOW);
    draw_text_raw(t, x, y, size as f32, c);
}

pub const SHADOW: Color = Color::new(0.08, 0.05, 0.16, 0.9);
pub const GLASS: Color = Color::new(0.62, 0.62, 0.84, 1.0);
pub const GLASS_HI: Color = Color::new(0.76, 0.76, 0.96, 1.0);

pub fn text_out_in(t: &str, r: Rect, y: f32, size: u16, c: Color, out: f32) {
    let w = measure_text(t, None, size, 1.0).width;
    text_out(t, r.x + (r.w - w) / 2.0, y, size, c, out);
}

pub fn wrap(t: &str, width: f32, size: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in t.split_whitespace() {
        let next = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if !cur.is_empty() && measure_text(&next, None, size, 1.0).width > width {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = next;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

pub fn ghost(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    kit().button.draw(r, r.h / 68.0 * 1.6, if h { GLASS_HI } else { GLASS });
    label_in(label, r, SMALL, if enabled { PALE } else { DIM });
}

pub fn plain(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    draw_rectangle(r.x, r.y, r.w, r.h, with_alpha(INK, if h { 0.16 } else { 0.08 }));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, with_alpha(INK, if h { 0.5 } else { 0.3 }));
    label_in(label, r, SMALL, if enabled { INK } else { DIM });
}

pub fn mint_button(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    let tint = if !enabled { Color::new(0.5, 0.55, 0.55, 1.0) } else if h { Color::new(0.75, 1.1, 1.0, 1.0) } else { Color::new(0.62, 1.0, 0.9, 1.0) };
    let y = if h && is_mouse_button_down(MouseButton::Left) { 2.0 } else { 0.0 };
    kit().button.draw(Rect::new(r.x, r.y + y, r.w, r.h), r.h / 68.0 * 1.6, tint);
    label_in(label, Rect::new(r.x, r.y + y, r.w, r.h), BODY, if enabled { WHITE } else { DIM });
}

pub fn card(r: Rect, on: bool) {
    let h = hovered(r);
    let lift = if h && !on { 3.0 } else { 0.0 };
    let r = Rect::new(r.x, r.y - lift, r.w, r.h);
    kit().panel.draw(r, 0.5, if on || h { GLASS_HI } else { GLASS });
    if on {
        draw_rectangle_lines(r.x - 5.0, r.y - 5.0, r.w + 10.0, r.h + 10.0, 4.0, GOLD);
    }
}

pub fn icon(t: &Texture2D, r: Rect) {
    draw_rectangle(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, INK);
    skill_art(t, r, WHITE);
}

const AZURE: [Color; 2] = [Color::new(0.81, 0.86, 0.98, 1.0), Color::new(0.35, 0.45, 0.79, 1.0)];

pub fn skill_art(t: &Texture2D, r: Rect, tint: Color) {
    if !crate::assets::zoomed_icon(t) {
        draw_texture_ex(t, r.x, r.y, tint, DrawTextureParams { dest_size: Some(r.size()), ..Default::default() });
        return;
    }
    let bands = 8;
    for i in 0..bands {
        let f = i as f32 / (bands - 1) as f32;
        let c = Color::new(AZURE[0].r + (AZURE[1].r - AZURE[0].r) * f, AZURE[0].g + (AZURE[1].g - AZURE[0].g) * f, AZURE[0].b + (AZURE[1].b - AZURE[0].b) * f, 1.0);
        let c = Color::new(c.r * tint.r, c.g * tint.g, c.b * tint.b, 1.0);
        draw_rectangle(r.x, r.y + r.h * i as f32 / bands as f32, r.w, r.h / bands as f32 + 0.5, c);
    }
    let side = t.width() / 1.42;
    let o = (t.width() - side) / 2.0;
    draw_texture_ex(t, r.x, r.y, tint, DrawTextureParams { dest_size: Some(r.size()), source: Some(Rect::new(o, o, side, side)), ..Default::default() });
}

pub fn portrait_box(kit: usize, r: Rect, bg: Color, flip: bool) {
    draw_rectangle(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, INK);
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    draw_rectangle(r.x, r.y, r.w, r.h * 0.4, with_alpha(WHITE, 0.15));
    let t = crate::assets::portrait(kit);
    let src = Rect::new(t.width() * 0.2, t.height() * 0.12, t.width() * 0.6, t.height() * 0.6);
    draw_texture_ex(t, r.x, r.y, WHITE, DrawTextureParams { dest_size: Some(r.size()), source: Some(src), flip_x: flip, ..Default::default() });
}

pub fn content_x(w: f32) -> f32 {
    ((screen_width() - w) / 2.0).max(16.0)
}

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

pub fn bar(r: Rect, frac: f32, lag: f32, fill: Color, right: bool) {
    kit().bar.draw(r, r.h / 22.0, WHITE);
    let (px, py) = (r.h * 0.5, r.h * 0.3);
    let inner = r.w - 2.0 * px;
    let w = inner * frac.clamp(0.0, 1.0);
    let x = if right { r.x + r.w - px - w } else { r.x + px };
    draw_rectangle(r.x + px, r.y + py, inner, r.h - 2.0 * py, Color::new(0.23, 0.2, 0.31, 1.0));
    let lw = inner * lag.clamp(0.0, 1.0);
    let lx = if right { r.x + r.w - px - lw } else { r.x + px };
    draw_rectangle(lx, r.y + py, lw, r.h - 2.0 * py, Color::new(1.0, 1.0, 1.0, 0.85));
    draw_rectangle(x, r.y + py, w, r.h - 2.0 * py, fill);
    draw_rectangle(x, r.y + py, w, (r.h - 2.0 * py) * 0.35, Color::new(1.0, 1.0, 1.0, 0.3));
}

pub fn panel_scaled(r: Rect, scale: f32) {
    if scale <= 0.5 {
        flat_panel(r);
        return;
    }
    kit().panel.draw(r, scale, GLASS);
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

pub fn button(r: Rect, label: &str, enabled: bool) {
    let h = enabled && hovered(r);
    let tint = if !enabled { Color::new(0.55, 0.55, 0.6, 1.0) } else if h { Color::new(1.1, 1.1, 1.1, 1.0) } else { WHITE };
    let y = if h && is_mouse_button_down(MouseButton::Left) { 2.0 } else { 0.0 };
    kit().button.draw(Rect::new(r.x, r.y + y, r.w, r.h), r.h / 68.0 * 1.6, tint);
    label_in(label, Rect::new(r.x, r.y + y, r.w, r.h), BODY, if enabled { WHITE } else { DIM });
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
        draw_text(label, r.x + 2.0, r.y - 8.0, 20.0, MINT);
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.09, 0.07, 0.16, 1.0));
        draw_rectangle(r.x, r.y, r.w, 3.0, Color::new(0.0, 0.0, 0.0, 0.35));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if focused { MINT } else { with_alpha(MINT, 0.35) });
        let shown = if self.secret { "*".repeat(self.value.len()) } else { self.value.clone() };
        let caret = if focused && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
        draw_text_raw(&format!("{shown}{caret}"), r.x + 16.0, r.y + r.h / 2.0 + 9.0, BODY as f32, CREAM);
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
