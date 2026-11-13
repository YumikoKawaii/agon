use macroquad::prelude::*;

const SPEED: f32 = 300.0;
const SIZE: f32 = 40.0;

struct Player {
    pos: Vec2,
    color: Color,
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

impl Player {
    fn update(&mut self, dt: f32) {
        let mut dir = Vec2::ZERO;
        if is_key_down(self.up) {
            dir.y -= 1.0;
        }
        if is_key_down(self.down) {
            dir.y += 1.0;
        }
        if is_key_down(self.left) {
            dir.x -= 1.0;
        }
        if is_key_down(self.right) {
            dir.x += 1.0;
        }
        self.pos += dir.normalize_or_zero() * SPEED * dt;
        self.pos.x = self.pos.x.clamp(0.0, screen_width() - SIZE);
        self.pos.y = self.pos.y.clamp(0.0, screen_height() - SIZE);
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, SIZE, SIZE, self.color);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Agon".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut players = [
        Player {
            pos: vec2(200.0, 340.0),
            color: SKYBLUE,
            up: KeyCode::W,
            down: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
        },
        Player {
            pos: vec2(1040.0, 340.0),
            color: ORANGE,
            up: KeyCode::Up,
            down: KeyCode::Down,
            left: KeyCode::Left,
            right: KeyCode::Right,
        },
    ];

    loop {
        let dt = get_frame_time();
        for p in players.iter_mut() {
            p.update(dt);
        }

        clear_background(Color::from_rgba(20, 20, 28, 255));
        for p in players.iter() {
            p.draw();
        }
        draw_text("AGON", 20.0, 40.0, 40.0, WHITE);

        next_frame().await;
    }
}
