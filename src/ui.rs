use crate::net::{call, watch, Next, Req, Watch, Wt};
use crate::player::KITS;
use crate::sprite::Sprites;
use macroquad::prelude::*;
use nanoserde::{DeJson, SerJson};

pub const BG: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const PANEL: Color = Color::new(0.13, 0.15, 0.2, 1.0);
const EDGE: Color = Color::new(0.35, 0.38, 0.5, 1.0);
const ACCENT: Color = Color::new(0.45, 1.0, 0.92, 1.0);
const DIM: Color = Color::new(0.6, 0.63, 0.72, 1.0);
const RED: Color = Color::new(1.0, 0.5, 0.5, 1.0);

pub enum Screen {
    Login,
    Menu,
    Select,
    Room,
    Lobby,
    Game,
}

#[derive(Clone, Copy)]
enum Call {
    Auth,
    Logout,
    Kit,
    Room,
    Ready,
    Leave,
    Endpoint,
}

#[derive(SerJson)]
struct Creds {
    username: String,
    password: String,
}

#[derive(SerJson)]
struct KitBody {
    kit: usize,
}

#[derive(SerJson)]
struct RoomBody {
    #[nserde(rename = "roomId")]
    room_id: String,
}

#[derive(SerJson)]
struct ReadyBody {
    #[nserde(rename = "roomId")]
    room_id: String,
    ready: bool,
}

#[derive(SerJson)]
struct MatchBody {
    #[nserde(rename = "matchId")]
    match_id: String,
}

#[derive(DeJson, Default, Clone)]
struct UserJson {
    #[nserde(default)]
    username: String,
    #[nserde(default)]
    kit: usize,
}

#[derive(DeJson)]
struct AuthResp {
    #[nserde(default)]
    token: String,
    #[nserde(default)]
    user: UserJson,
}

#[derive(DeJson, Default, Clone)]
pub struct Seat {
    #[nserde(default)]
    pub username: String,
    #[nserde(default)]
    pub kit: usize,
    #[nserde(default)]
    pub ready: bool,
}

#[derive(DeJson, Default, Clone)]
pub struct RoomJson {
    #[nserde(default)]
    pub id: String,
    #[nserde(default)]
    pub host: Seat,
    #[nserde(default)]
    pub guest: Seat,
    #[nserde(default)]
    pub state: String,
    #[nserde(default, rename = "matchId")]
    pub match_id: String,
}

#[derive(DeJson)]
struct RoomResp {
    #[nserde(default)]
    room: RoomJson,
}

#[derive(DeJson)]
struct EndpointResp {
    #[nserde(default)]
    url: String,
    #[nserde(default, rename = "certHash")]
    cert_hash: String,
    #[nserde(default)]
    ticket: String,
    #[nserde(default)]
    seat: usize,
}

#[derive(DeJson)]
struct ErrResp {
    #[nserde(default)]
    message: String,
}

pub struct Online {
    pub wt: Wt,
    pub seat: usize,
    pub foe: String,
    pub foe_kit: usize,
    pub started: bool,
    pub result: Option<(String, f32)>,
}

pub struct Ui {
    pub screen: Screen,
    pub user: String,
    pass: String,
    token: String,
    focus: usize,
    error: String,
    pub kit: usize,
    pick: usize,
    pub room: Option<RoomJson>,
    room_input: String,
    pending: Option<(Call, Req)>,
    watch: Option<Watch>,
    pub online: Option<Online>,
    pub quit: bool,
}

fn hover(r: Rect) -> bool {
    r.contains(Vec2::from(mouse_position()))
}

pub fn center_text(t: &str, y: f32, size: u16, c: Color) {
    let w = measure_text(t, None, size, 1.0).width;
    draw_text(t, (screen_width() - w) / 2.0, y, size as f32, c);
}

fn button(r: Rect, label: &str, enabled: bool) -> bool {
    let h = enabled && hover(r);
    draw_rectangle(r.x, r.y, r.w, r.h, if h { EDGE } else { PANEL });
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if h { ACCENT } else { EDGE });
    let m = measure_text(label, None, 26, 1.0);
    draw_text(label, r.x + (r.w - m.width) / 2.0, r.y + r.h / 2.0 + 9.0, 26.0, if enabled { WHITE } else { DIM });
    h && is_mouse_button_pressed(MouseButton::Left)
}

fn field(r: Rect, label: &str, value: &str, mask: bool, focused: bool) -> bool {
    draw_text(label, r.x, r.y - 8.0, 20.0, DIM);
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if focused { ACCENT } else { EDGE });
    let shown = if mask { "*".repeat(value.len()) } else { value.to_string() };
    let caret = if focused && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
    draw_text(&format!("{shown}{caret}"), r.x + 12.0, r.y + r.h / 2.0 + 9.0, 26.0, WHITE);
    hover(r) && is_mouse_button_pressed(MouseButton::Left)
}

fn type_into(s: &mut String, max: usize, any: bool) {
    while let Some(c) = get_char_pressed() {
        if (any && c.is_ascii_graphic() || c.is_ascii_alphanumeric() || c == '_' || c == '-') && s.len() < max {
            s.push(c);
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        s.pop();
    }
}

fn mid(w: f32, y: f32, h: f32) -> Rect {
    Rect::new((screen_width() - w) / 2.0, y, w, h)
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            screen: Screen::Login,
            user: String::new(),
            pass: String::new(),
            token: String::new(),
            focus: 0,
            error: String::new(),
            kit: 0,
            pick: 0,
            room: None,
            room_input: String::new(),
            pending: None,
            watch: None,
            online: None,
            quit: false,
        }
    }

    fn busy(&self) -> bool {
        self.pending.is_some()
    }

    fn call(&mut self, c: Call, rpc: &str, body: String) {
        self.error.clear();
        self.pending = Some((c, call(rpc, &body, &self.token)));
    }

    fn room_id(&self) -> String {
        self.room.as_ref().map(|r| r.id.clone()).unwrap_or_default()
    }

    pub fn is_host(&self) -> bool {
        self.room.as_ref().is_some_and(|r| r.host.username == self.user)
    }

    fn open_watch(&mut self) {
        let body = RoomBody { room_id: self.room_id() }.serialize_json();
        self.watch = Some(watch("RoomService/WatchRoom", &body, &self.token));
    }

    pub fn leave(&mut self) {
        let body = RoomBody { room_id: self.room_id() }.serialize_json();
        self.call(Call::Leave, "RoomService/LeaveRoom", body);
        self.watch = None;
        self.online = None;
        self.room = None;
        self.screen = Screen::Menu;
    }

    fn on_room(&mut self, r: RoomJson) {
        match r.state.as_str() {
            "ROOM_STATE_CLOSED" => {
                self.watch = None;
                self.online = None;
                self.room = None;
                self.error = "room closed".to_string();
                self.screen = Screen::Menu;
                return;
            }
            "ROOM_STATE_PLAYING" if self.online.is_none() && !self.busy() && !r.match_id.is_empty() => {
                let body = MatchBody { match_id: r.match_id.clone() }.serialize_json();
                self.call(Call::Endpoint, "MatchService/GetEndpoint", body);
            }
            _ => {}
        }
        self.room = Some(r);
    }

    fn poll_watch(&mut self) {
        let Some(w) = self.watch.as_mut() else { return };
        let mut msgs = Vec::new();
        let mut closed = false;
        loop {
            match w.next() {
                Next::Msg(m) => msgs.push(m),
                Next::Wait => break,
                Next::Closed => {
                    closed = true;
                    break;
                }
            }
        }
        for m in msgs {
            if let Ok(r) = RoomResp::deserialize_json(&m) {
                self.on_room(r.room);
            }
        }
        if closed && self.room.is_some() {
            self.open_watch();
        }
    }

    pub fn tick(&mut self) {
        self.poll();
        self.poll_watch();
    }

    fn poll(&mut self) {
        let Some((call, req)) = self.pending.as_mut() else { return };
        let Some((status, body)) = req.poll() else { return };
        let call = *call;
        self.pending = None;
        if !(200..300).contains(&status) {
            self.error = match ErrResp::deserialize_json(&body) {
                Ok(e) if !e.message.is_empty() => e.message,
                _ if status < 0 => "server unreachable".to_string(),
                _ => format!("error {status}"),
            };
            if status == 401 && !matches!(call, Call::Auth) {
                self.signed_out();
            }
            return;
        }
        match call {
            Call::Auth => {
                if let Ok(a) = AuthResp::deserialize_json(&body) {
                    self.token = a.token;
                    self.user = a.user.username;
                    self.kit = a.user.kit.min(KITS.len() - 1);
                    self.pass.clear();
                    self.screen = Screen::Menu;
                }
            }
            Call::Logout => self.signed_out(),
            Call::Kit => {
                self.kit = self.pick;
                self.screen = Screen::Menu;
            }
            Call::Room => {
                if let Ok(r) = RoomResp::deserialize_json(&body) {
                    self.room = Some(r.room);
                    self.screen = Screen::Lobby;
                    self.open_watch();
                }
            }
            Call::Ready => {
                if let Ok(r) = RoomResp::deserialize_json(&body) {
                    self.on_room(r.room);
                }
            }
            Call::Leave => {}
            Call::Endpoint => {
                if let (Ok(e), Some(r)) = (EndpointResp::deserialize_json(&body), self.room.clone()) {
                    let foe = if e.seat == 0 { r.guest } else { r.host };
                    let url = format!("{}?ticket={}", e.url, e.ticket);
                    self.online = Some(Online {
                        wt: Wt::open(&url, &e.cert_hash),
                        seat: e.seat,
                        foe: foe.username,
                        foe_kit: foe.kit.min(KITS.len() - 1),
                        started: false,
                        result: None,
                    });
                    self.screen = Screen::Game;
                }
            }
        }
    }

    fn signed_out(&mut self) {
        self.token.clear();
        self.user.clear();
        self.focus = 0;
        self.watch = None;
        self.online = None;
        self.room = None;
        self.screen = Screen::Login;
    }

    pub fn draw(&mut self, skins: &[Sprites; 3]) {
        self.tick();
        clear_background(BG);
        match self.screen {
            Screen::Login => self.login(),
            Screen::Menu => self.menu(),
            Screen::Select => self.select(skins),
            Screen::Room => self.room_screen(),
            Screen::Lobby => self.lobby(skins),
            Screen::Game => {}
        }
        center_text(&self.error, screen_height() - 70.0, 22, RED);
        if self.busy() {
            center_text("...", screen_height() - 100.0, 28, DIM);
        }
    }

    fn login(&mut self) {
        center_text("AGON", 160.0, 96, ACCENT);
        center_text("sign in or create an account", 200.0, 24, DIM);
        let u = mid(360.0, 260.0, 48.0);
        let p = mid(360.0, 350.0, 48.0);
        if field(u, "username", &self.user, false, self.focus == 0) {
            self.focus = 0;
        }
        if field(p, "password", &self.pass, true, self.focus == 1) {
            self.focus = 1;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.focus = 1 - self.focus;
        }
        if self.focus == 0 {
            type_into(&mut self.user, 16, false);
        } else {
            type_into(&mut self.pass, 64, true);
        }
        let ok = !self.busy();
        let r = mid(360.0, 430.0, 52.0);
        let sign = button(Rect::new(r.x, r.y, 174.0, r.h), "Sign in", ok) || (ok && is_key_pressed(KeyCode::Enter));
        let reg = button(Rect::new(r.x + 186.0, r.y, 174.0, r.h), "Register", ok);
        if sign || reg {
            let body = Creds { username: self.user.clone(), password: self.pass.clone() }.serialize_json();
            self.call(Call::Auth, if reg { "AuthService/Register" } else { "AuthService/Login" }, body);
        }
    }

    fn menu(&mut self) {
        center_text("AGON", 160.0, 96, ACCENT);
        center_text(&format!("signed in as {}   |   {}", self.user, KITS[self.kit].name), 205.0, 24, DIM);
        let ok = !self.busy();
        if button(mid(320.0, 250.0, 56.0), "Play online", ok) {
            self.room_input.clear();
            self.error.clear();
            self.screen = Screen::Room;
        }
        if button(mid(320.0, 316.0, 56.0), "Practice", ok) {
            self.error.clear();
            self.screen = Screen::Game;
        }
        if button(mid(320.0, 382.0, 56.0), "Characters", ok) {
            self.pick = self.kit;
            self.error.clear();
            self.screen = Screen::Select;
        }
        if button(mid(320.0, 448.0, 56.0), "Sign out", ok) {
            self.call(Call::Logout, "AuthService/Logout", String::new());
        }
        if button(mid(320.0, 514.0, 56.0), "Quit", ok) {
            self.quit = true;
        }
    }

    fn select(&mut self, skins: &[Sprites; 3]) {
        center_text("Choose your character", 110.0, 48, WHITE);
        let w = 260.0;
        let gap = 30.0;
        let x0 = (screen_width() - (w * 3.0 + gap * 2.0)) / 2.0;
        for (i, kit) in KITS.iter().enumerate() {
            let r = Rect::new(x0 + i as f32 * (w + gap), 160.0, w, 380.0);
            let chosen = self.pick == i;
            draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, if chosen { 4.0 } else { 2.0 }, if chosen || hover(r) { ACCENT } else { EDGE });
            let tex = skins[i].frame(crate::anim::State::Idle, 2, (get_time() * 60.0) as u32, 1);
            let size = vec2(tex.width(), tex.height()) * 3.0;
            draw_texture_ex(tex, r.x + (w - size.x) / 2.0, r.y + 20.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
            let m = measure_text(kit.name, None, 28, 1.0);
            draw_text(kit.name, r.x + (w - m.width) / 2.0, r.y + 250.0, 28.0, WHITE);
            let hp = format!("HP {}", kit.hp);
            let m = measure_text(&hp, None, 22, 1.0);
            draw_text(&hp, r.x + (w - m.width) / 2.0, r.y + 285.0, 22.0, DIM);
            if !self.busy() && hover(r) && is_mouse_button_pressed(MouseButton::Left) {
                self.pick = i;
            }
        }
        let ok = !self.busy();
        if button(mid(320.0, 580.0, 56.0), "Confirm", ok) {
            self.call(Call::Kit, "UserService/SetKit", KitBody { kit: self.pick }.serialize_json());
        }
        if ok && is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
        }
    }

    fn room_screen(&mut self) {
        center_text("Room", 140.0, 56, WHITE);
        center_text(&format!("playing as {}", KITS[self.kit].name), 180.0, 24, DIM);
        let ok = !self.busy();
        if button(mid(360.0, 230.0, 56.0), "Create room", ok) {
            self.call(Call::Room, "RoomService/CreateRoom", String::new());
        }
        center_text("or join by id", 335.0, 22, DIM);
        field(mid(360.0, 360.0, 48.0), "room id", &self.room_input, false, true);
        type_into(&mut self.room_input, 12, false);
        let join = button(mid(360.0, 430.0, 52.0), "Join", ok) || (ok && is_key_pressed(KeyCode::Enter));
        if join && !self.room_input.is_empty() {
            let body = RoomBody { room_id: self.room_input.clone() }.serialize_json();
            self.call(Call::Room, "RoomService/JoinRoom", body);
        }
        if button(mid(360.0, 510.0, 52.0), "Back", ok) || (ok && is_key_pressed(KeyCode::Escape)) {
            self.screen = Screen::Menu;
        }
    }

    fn lobby(&mut self, skins: &[Sprites; 3]) {
        let Some(r) = self.room.clone() else {
            self.screen = Screen::Menu;
            return;
        };
        center_text(&format!("Room {}", r.id), 110.0, 56, WHITE);
        center_text("share this id with your opponent", 145.0, 22, DIM);
        let w = 300.0;
        let x0 = (screen_width() - (w * 2.0 + 40.0)) / 2.0;
        for (i, seat) in [&r.host, &r.guest].into_iter().enumerate() {
            let b = Rect::new(x0 + i as f32 * (w + 40.0), 180.0, w, 300.0);
            draw_rectangle(b.x, b.y, b.w, b.h, PANEL);
            draw_rectangle_lines(b.x, b.y, b.w, b.h, 2.0, if seat.ready { ACCENT } else { EDGE });
            let role = if i == 0 { "host" } else { "guest" };
            draw_text(role, b.x + 14.0, b.y + 28.0, 20.0, DIM);
            if seat.username.is_empty() {
                let m = measure_text("waiting...", None, 26, 1.0);
                draw_text("waiting...", b.x + (w - m.width) / 2.0, b.y + 160.0, 26.0, DIM);
                continue;
            }
            let kit = seat.kit.min(KITS.len() - 1);
            let tex = skins[kit].frame(crate::anim::State::Idle, 2, (get_time() * 60.0) as u32, 1);
            let size = vec2(tex.width(), tex.height()) * 2.5;
            draw_texture_ex(tex, b.x + (w - size.x) / 2.0, b.y + 30.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
            let name = format!("{}  ({})", seat.username, KITS[kit].name);
            let m = measure_text(&name, None, 22, 1.0);
            draw_text(&name, b.x + (w - m.width) / 2.0, b.y + 240.0, 22.0, WHITE);
            let st = if seat.ready { "READY" } else { "not ready" };
            let m = measure_text(st, None, 24, 1.0);
            draw_text(st, b.x + (w - m.width) / 2.0, b.y + 275.0, 24.0, if seat.ready { ACCENT } else { DIM });
        }
        let ok = !self.busy() && r.state == "ROOM_STATE_WAITING";
        let mine = if self.is_host() { r.host.ready } else { r.guest.ready };
        if button(mid(320.0, 510.0, 56.0), if mine { "Unready" } else { "Ready" }, ok) {
            let body = ReadyBody { room_id: r.id.clone(), ready: !mine }.serialize_json();
            self.call(Call::Ready, "RoomService/SetReady", body);
        }
        if button(mid(320.0, 580.0, 52.0), "Leave", !self.busy()) || (!self.busy() && is_key_pressed(KeyCode::Escape)) {
            self.leave();
        }
    }
}
