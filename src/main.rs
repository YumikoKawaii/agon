mod anim;
mod player;

use anim::*;
use macroquad::prelude::*;
use player::*;

const S: f32 = 3.0;
const POST_H: f32 = 14.0;
const TILE: f32 = 20.0;

thread_local! {
    static CAM: std::cell::Cell<Vec2> = const { std::cell::Cell::new(Vec2::ZERO) };
}

fn proj(v: Vec2) -> Vec2 {
    v * S - CAM.with(|c| c.get())
}

fn update_camera(players: &[Player; 2]) {
    let mid = (players[0].pos + players[1].pos) / 2.0 * S;
    let half = vec2(screen_width(), screen_height()) / 2.0;
    let lo = vec2(-40.0, -POST_H * S - 40.0);
    let hi = vec2(ARENA * S + 40.0, ARENA * S + 40.0) - half * 2.0;
    let target = mid - half;
    CAM.with(|c| c.set(vec2(target.x.clamp(lo.x, hi.x.max(lo.x)), target.y.clamp(lo.y, hi.y.max(lo.y)))));
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Agon".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn resolve_push(players: &mut [Player; 2]) {
    let d = players[1].pos - players[0].pos;
    let len = d.length().max(0.01);
    let min = RADIUS * 2.0;
    if len < min {
        let n = d / len * (min - len) / 2.0;
        players[0].pos -= n;
        players[1].pos += n;
        players[0].clamp();
        players[1].clamp();
    }
}

fn resolve_hits(players: &mut [Player; 2], anims: &Anims) {
    let mut hits = [false; 2];
    for a in 0..2 {
        let p = &players[a];
        let hurt = players[1 - a].hurtboxes(anims);
        hits[a] = !p.has_hit
            && p.current(anims)
                .hit
                .iter()
                .any(|s| hurt.iter().any(|(c, r)| p.hits(s, *c, *r)));
    }
    for a in 0..2 {
        if hits[a] {
            players[a].has_hit = true;
            let dir = players[a].facing();
            players[1 - a].take_hit(dir, 10);
        }
    }
    if players.iter().any(|p| p.hp <= 0) {
        players.iter_mut().for_each(|p| p.respawn());
    }
}

fn draw_floor() {
    let n = (ARENA / TILE) as i32;
    for i in 0..n {
        for j in 0..n {
            let p = proj(vec2(i as f32 * TILE, j as f32 * TILE));
            let c = if (i + j) % 2 == 0 {
                Color::from_rgba(51, 56, 74, 255)
            } else {
                Color::from_rgba(58, 63, 82, 255)
            };
            draw_rectangle(p.x, p.y, TILE * S, TILE * S, c);
        }
    }
}

fn draw_cage(front: bool) {
    let col = Color::from_rgba(138, 144, 166, 255);
    let n = (ARENA / TILE) as i32;
    let mut posts = Vec::new();
    for k in 0..=n {
        let t = k as f32 * TILE;
        posts.extend([vec2(t, 0.0), vec2(t, ARENA), vec2(0.0, t), vec2(ARENA, t)]);
    }
    for w in posts {
        if (w.y == ARENA) != front {
            continue;
        }
        let p = proj(w);
        draw_line(p.x, p.y, p.x, p.y - POST_H * S, 2.0, col);
    }
    let edges: &[(Vec2, Vec2)] = if front {
        &[(vec2(0.0, ARENA), vec2(ARENA, ARENA))]
    } else {
        &[
            (vec2(0.0, 0.0), vec2(ARENA, 0.0)),
            (vec2(0.0, 0.0), vec2(0.0, ARENA)),
            (vec2(ARENA, 0.0), vec2(ARENA, ARENA)),
        ]
    };
    for (a, b) in edges {
        let (a, b) = (proj(*a), proj(*b));
        for h in [POST_H, POST_H / 2.0] {
            draw_line(a.x, a.y - h * S, b.x, b.y - h * S, 2.0, col);
        }
    }
}

fn draw_cone(p: &Player, r: f32, half: f32, fill: Color) {
    let c = proj(p.pos);
    let steps = 10;
    for i in 0..steps {
        let a0 = p.ang - half + 2.0 * half * i as f32 / steps as f32;
        let a1 = p.ang - half + 2.0 * half * (i + 1) as f32 / steps as f32;
        let v0 = proj(p.pos + Vec2::from_angle(a0) * r);
        let v1 = proj(p.pos + Vec2::from_angle(a1) * r);
        draw_triangle(c, v0, v1, fill);
    }
}

fn draw_player(p: &Player, anims: &Anims, debug: bool) {
    let s = proj(p.pos);
    draw_circle(s.x, s.y, RADIUS * S, Color::new(0.0, 0.0, 0.0, 0.45));

    let attack = anims.get(State::Attack);
    if p.state == State::Attack {
        if let Some(Shape::Cone { r, half }) = attack.frames[1].hit.first() {
            if p.frame == 0 {
                draw_cone(p, *r, *half, Color::new(1.0, 0.47, 0.35, 0.18));
            }
        }
    }
    for shape in &p.current(anims).hit {
        match *shape {
            Shape::Cone { r, half } => draw_cone(p, r, half, Color::new(1.0, 0.27, 0.2, 0.45)),
            Shape::Circle(c) => {
                let at = proj(p.place(&c));
                draw_circle(at.x, at.y, c.r * S, Color::new(1.0, 0.27, 0.2, 0.45));
            }
        }
    }
    if debug {
        for (c, r) in p.hurtboxes(anims) {
            let c = proj(c);
            draw_circle(c.x, c.y, r * S, Color::new(0.35, 0.82, 1.0, 0.2));
            draw_circle_lines(c.x, c.y, r * S, 1.5, Color::from_rgba(90, 209, 255, 255));
        }
    }

    let col = if p.state == State::Hit { WHITE } else { p.color };
    let px = |x: f32, y: f32, w: f32, h: f32, c: Color| {
        draw_rectangle(s.x.round() + x * S, s.y.round() + y * S, w * S, h * S, c);
    };
    let dark = Color::from_rgba(26, 28, 36, 255);
    let f = p.facing();
    if p.state == State::Roll {
        px(-6.0, -11.0, 12.0, 10.0, col);
        px(-3.0, -8.0, 6.0, 4.0, dark);
    } else {
        px(-4.0, -13.0, 8.0, 11.0, col);
        px(-3.0, -18.0, 6.0, 5.0, col);
        px(-4.0, -2.0, 3.0, 2.0, dark);
        px(1.0, -2.0, 3.0, 2.0, dark);
    }
    if p.state != State::Roll && f.y > -0.3 {
        let ex = (f.x * 2.0).round();
        px(ex - 1.0, -16.0, 1.0, 1.0, dark);
        px(ex + 1.0, -16.0, 1.0, 1.0, dark);
    }
    if p.state == State::Attack && p.frame == 1 {
        let a = vec2(s.x, s.y - 8.5 * S);
        let b = a + f * 12.0 * S;
        draw_line(a.x, a.y, b.x, b.y, S, Color::from_rgba(233, 237, 245, 255));
    }
    if debug {
        let label = format!("{:?} {} d{}", p.state, p.frame, p.dir8());
        draw_text(&label, s.x - 40.0, s.y - 22.0 * S, 20.0, WHITE);
    }
}

fn draw_hud(players: &[Player; 2]) {
    let w = 300.0;
    for (i, p) in players.iter().enumerate() {
        let x = if i == 0 { 40.0 } else { screen_width() - 40.0 - w };
        draw_rectangle(x, 24.0, w, 14.0, Color::from_rgba(34, 38, 52, 255));
        let fill = w * p.hp.max(0) as f32 / 100.0;
        let fx = if i == 0 { x } else { x + w - fill };
        draw_rectangle(fx, 24.0, fill, 14.0, p.color);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let anims = Anims::new();
    let mut players = [
        Player::new(
            vec2(100.0, 200.0),
            0.0,
            SKYBLUE,
            Controls {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
                attack: KeyCode::F,
                dodge: KeyCode::LeftShift,
            },
        ),
        Player::new(
            vec2(300.0, 200.0),
            std::f32::consts::PI,
            ORANGE,
            Controls {
                up: KeyCode::Up,
                down: KeyCode::Down,
                left: KeyCode::Left,
                right: KeyCode::Right,
                attack: KeyCode::Enter,
                dodge: KeyCode::RightShift,
            },
        ),
    ];
    let mut pending = [Input::default(); 2];
    let mut acc = 0.0;
    let mut debug = true;

    loop {
        if is_key_pressed(KeyCode::Tab) {
            debug = !debug;
        }
        for i in 0..2 {
            let r = players[i].controls.read();
            pending[i].dir = r.dir;
            pending[i].attack |= r.attack;
            pending[i].dodge |= r.dodge;
        }

        acc += get_frame_time().min(0.25);
        while acc >= DT {
            for i in 0..2 {
                players[i].step(pending[i], &anims);
                pending[i].attack = false;
                pending[i].dodge = false;
            }
            resolve_push(&mut players);
            resolve_hits(&mut players, &anims);
            acc -= DT;
        }

        update_camera(&players);
        clear_background(Color::from_rgba(13, 15, 22, 255));
        draw_floor();
        draw_cage(false);
        let mut order = [0, 1];
        order.sort_by(|a, b| players[*a].pos.y.total_cmp(&players[*b].pos.y));
        for i in order {
            draw_player(&players[i], &anims, debug);
        }
        draw_cage(true);
        draw_hud(&players);
        draw_text(
            "P1: WASD, F attack, LShift roll    P2: Arrows, Enter attack, RShift roll    Tab: boxes",
            40.0,
            screen_height() - 20.0,
            24.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
