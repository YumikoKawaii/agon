mod anim;
mod player;
mod skills;
mod sprite;

use anim::*;
use macroquad::prelude::*;
use player::*;
use skills::*;
use sprite::*;

const S: f32 = 3.0;
const SPRITE: f32 = 0.5;
const FOOT: f32 = 9.0;
const POST_H: f32 = 14.0;
const TILE: f32 = 20.0;

thread_local! {
    static CAM: std::cell::Cell<Vec2> = const { std::cell::Cell::new(Vec2::ZERO) };
}

fn proj(v: Vec2) -> Vec2 {
    v * S - CAM.with(|c| c.get())
}

fn mouse_world() -> Vec2 {
    (Vec2::from(mouse_position()) + CAM.with(|c| c.get()) + vec2(0.0, 9.0 * S)) / S
}

fn update_camera(players: &[Player; 2], shake: f32) {
    let mid = players[0].pos * S;
    let half = vec2(screen_width(), screen_height()) / 2.0;
    let lo = vec2(-40.0, -POST_H * S - 40.0);
    let hi = vec2(ARENA * S + 40.0, ARENA * S + 40.0) - half * 2.0;
    let jitter = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * shake;
    let target = mid - half + jitter;
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
        players.iter_mut().filter(|p| p.hp <= 0).for_each(|p| p.respawn());
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

fn draw_player(p: &Player, anims: &Anims, sprites: &Sprites, debug: bool) {
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

    let total: u32 = if p.state == State::Attack {
        p.cast_ticks
    } else {
        anims.get(p.state).frames.iter().map(|f| f.ticks).sum()
    };
    for (g, t, d) in &p.ghosts {
        let gs = proj(*g);
        let gt = sprites.frame(State::Run, *d, 0, 1);
        let gsize = vec2(gt.width(), gt.height()) * SPRITE * S;
        draw_texture_ex(
            gt,
            (gs.x - gsize.x / 2.0).round(),
            (gs.y - gsize.y / 2.0 - FOOT * S).round(),
            Color::new(0.45, 1.0, 0.92, (t / 0.3).min(1.0) * 0.55),
            DrawTextureParams { dest_size: Some(gsize), ..Default::default() },
        );
    }
    if p.untargetable {
        return;
    }
    let tex = sprites.frame(p.state, p.dir8(), p.age, total);
    let size = vec2(tex.width(), tex.height()) * SPRITE * S;
    let tint = if p.state == State::Hit && p.age < 4 { Color::new(1.0, 0.6, 0.6, 1.0) } else { p.tint };
    draw_texture_ex(
        tex,
        (s.x - size.x / 2.0).round(),
        (s.y - size.y / 2.0 - FOOT * S).round(),
        tint,
        DrawTextureParams { dest_size: Some(size), ..Default::default() },
    );
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
        let fill = w * p.hp.max(0) as f32 / p.max_hp as f32;
        let fx = if i == 0 { x } else { x + w - fill };
        draw_rectangle(fx, 24.0, fill, 14.0, p.color);
        let label = format!("{}  {}/{}", KITS[p.kit].name, p.hp.max(0), p.max_hp);
        let lx = if i == 0 { x } else { x + w - measure_text(&label, None, 18, 1.0).width };
        draw_text(&label, lx, 18.0, 18.0, LIGHTGRAY);
        let skills = p.skills();
        for (k, name) in ["LMB", "Q", "E", "R"].iter().enumerate() {
            let bx = if i == 0 { x + k as f32 * 52.0 } else { x + w - 48.0 - k as f32 * 52.0 };
            draw_rectangle(bx, 44.0, 48.0, 22.0, Color::from_rgba(34, 38, 52, 255));
            let wind = skills[k] == Skill::WindStep;
            let ready = if wind { p.charges > 0 } else { p.cd[k] <= 0.0 };
            let frac = if wind && p.charges >= WIND_CHARGES { 0.0 } else { p.cd[k] / skills[k].cooldown() };
            let col = if ready { ICE } else { Color::from_rgba(70, 76, 98, 255) };
            draw_rectangle(bx, 44.0, 48.0 * (1.0 - frac), 22.0, col);
            let text = if wind { format!("{} {}", name, p.charges) } else { name.to_string() };
            draw_text(&text, bx + 6.0, 61.0, 20.0, Color::from_rgba(20, 20, 28, 255));
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let anims = Anims::new();
    let sword_tex = load_texture("assets/sword/spirit_sword.png").await.unwrap();
    sword_tex.set_filter(FilterMode::Nearest);
    let skins = [
        Sprites::load("assets/elf").await,
        Sprites::load("assets/witch").await,
        Sprites::load("assets/sword").await,
    ];
    let mut skin = [0, 1];
    let mut players = [
        Player::new(vec2(120.0, 200.0), 0.0, SKYBLUE, false, 0),
        Player::new(vec2(280.0, 200.0), std::f32::consts::PI, ORANGE, true, 1),
    ];
    let mut pending = Input::default();
    let mut acc = 0.0;
    let mut world = World::default();
    let mut debug = false;

    loop {
        if is_key_pressed(KeyCode::Tab) {
            debug = !debug;
        }
        for (k, n) in [(KeyCode::Key1, 0), (KeyCode::Key2, 1), (KeyCode::Key3, 2)] {
            if is_key_pressed(k) && skin[0] != n {
                skin[0] = n;
                players[0].set_kit(n);
                world = World::default();
                players[1].untargetable = false;
            }
        }
        let r = read_input(mouse_world());
        pending.dir = r.dir;
        pending.aim = r.aim;
        pending.slot = r.slot.or(pending.slot);
        pending.dodge |= r.dodge;

        acc += get_frame_time().min(0.25);
        while acc >= DT {
            for i in 0..2 {
                let input = if players[i].dummy { Input::default() } else { pending };
                players[i].step(input, &anims);
                world.cast(&mut players, i, &anims);
            }
            pending.slot = None;
            pending.dodge = false;
            let to = players[0].pos - players[1].pos;
            players[1].ang = to.y.atan2(to.x);
            resolve_push(&mut players);
            world.update(&mut players, &anims);
            resolve_hits(&mut players, &anims);
            acc -= DT;
        }

        update_camera(&players, world.shake);
        clear_background(Color::from_rgba(13, 15, 22, 255));
        draw_floor();
        draw_cage(false);
        world.draw_ground();
        let mut order = [0, 1];
        order.sort_by(|a, b| players[*a].pos.y.total_cmp(&players[*b].pos.y));
        for i in order {
            draw_player(&players[i], &anims, &skins[skin[i]], debug);
        }
        world.draw_air(&players, &sword_tex);
        draw_cage(true);
        let me = proj(players[0].pos);
        let aim = proj(mouse_world());
        let elf = players[0].kit != 2;
        if is_key_down(KeyCode::R) {
            let (r, c) = if elf { (JUDGE_RANGE, Color::new(1.0, 0.86, 0.45, 0.35)) } else { (FORM_RANGE, Color::new(0.45, 1.0, 0.92, 0.35)) };
            draw_circle_lines(me.x, me.y, r * S, 1.5, c);
        }
        if elf && is_key_down(KeyCode::Q) {
            let d = (mouse_world() - players[0].pos).normalize_or(Vec2::X);
            let e = proj(players[0].pos + d * RAY_LEN);
            draw_line(me.x, me.y - 9.0 * S, e.x, e.y - 9.0 * S, 2.0, Color::new(0.55, 0.85, 1.0, 0.3));
        }
        draw_line(me.x, me.y - 9.0 * S, aim.x, aim.y - 9.0 * S, 1.0, Color::new(1.0, 1.0, 1.0, 0.15));
        let m = Vec2::from(mouse_position());
        draw_circle_lines(m.x, m.y, 8.0, 1.5, WHITE);
        draw_circle(m.x, m.y, 2.0, WHITE);
        world.draw_flash();
        draw_hud(&players);
        draw_text(
            "WASD move   Mouse aim   RMB roll   1/2/3 character   Tab boxes",
            40.0,
            screen_height() - 46.0,
            24.0,
            LIGHTGRAY,
        );
        draw_text(
            KITS[players[0].kit].hint,
            40.0,
            screen_height() - 20.0,
            24.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
