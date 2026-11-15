mod anim;
mod player;
mod sprite;

use anim::*;
use macroquad::prelude::*;
use player::*;
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

fn update_camera(players: &[Player; 2]) {
    let mid = players[0].pos * S;
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
        players.iter_mut().filter(|p| p.hp <= 0).for_each(|p| p.respawn());
    }
}

const STAR_R: f32 = 40.0;
const STAR_T: f32 = 0.8;
const STAR_RANGE: f32 = 110.0;
const MINT: Color = Color::new(0.66, 0.96, 0.75, 1.0);
const LAV: Color = Color::new(0.85, 0.62, 0.95, 1.0);
const GOLD: Color = Color::new(1.0, 0.86, 0.45, 1.0);

struct Shot {
    pos: Vec2,
    vel: Vec2,
    r: f32,
    dmg: i32,
    owner: usize,
    life: f32,
    big: bool,
}

struct Star {
    pos: Vec2,
    t: f32,
    owner: usize,
}

enum FxKind {
    Ring(f32, Color),
    Trail(Vec2),
}

struct Fx {
    pos: Vec2,
    t: f32,
    max: f32,
    kind: FxKind,
}

#[derive(Default)]
struct World {
    shots: Vec<Shot>,
    stars: Vec<Star>,
    fx: Vec<Fx>,
}

impl World {
    fn ring(&mut self, pos: Vec2, r: f32, c: Color, max: f32) {
        self.fx.push(Fx { pos, t: max, max, kind: FxKind::Ring(r, c) });
    }

    fn cast(&mut self, p: &mut Player, i: usize) {
        let Some(k) = p.cast.take() else { return };
        let f = p.facing();
        let o = p.pos + f * 8.0;
        match k {
            Skill::Bolt => self.shots.push(Shot { pos: o, vel: f * 240.0, r: 3.0, dmg: 6, owner: i, life: 0.7, big: false }),
            Skill::Orb => self.shots.push(Shot { pos: o, vel: f * 80.0, r: 8.0, dmg: 16, owner: i, life: 3.0, big: true }),
            Skill::Blink => {
                if let Some(from) = p.blink_from.take() {
                    self.fx.push(Fx { pos: from, t: 0.35, max: 0.35, kind: FxKind::Trail(p.pos) });
                    self.ring(from, 12.0, LAV, 0.3);
                    self.ring(p.pos, 12.0, MINT, 0.3);
                }
            }
            Skill::Star => {
                let m = STAR_R * 0.5;
                let at = (p.pos + (p.target - p.pos).clamp_length_max(STAR_RANGE)).clamp(vec2(m, m), vec2(ARENA - m, ARENA - m));
                self.stars.push(Star { pos: at, t: STAR_T, owner: i });
            }
        }
    }

    fn update(&mut self, players: &mut [Player; 2], anims: &Anims) {
        let mut rings = Vec::new();
        self.shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            let t = 1 - s.owner;
            let hit = players[t].hurtboxes(anims).iter().any(|(c, r)| c.distance(s.pos) < r + s.r);
            if hit {
                players[t].take_hit(s.vel.normalize_or_zero(), s.dmg);
                rings.push((s.pos, s.r * 2.5, if s.big { LAV } else { MINT }));
            }
            let inside = s.pos.cmpge(Vec2::ZERO).all() && s.pos.cmple(Vec2::splat(ARENA)).all();
            !hit && s.life > 0.0 && inside
        });
        self.stars.retain_mut(|s| {
            s.t -= DT;
            if s.t > 0.0 {
                return true;
            }
            let t = 1 - s.owner;
            let d = players[t].pos - s.pos;
            if !players[t].hurtboxes(anims).is_empty() && d.length() < STAR_R + RADIUS {
                players[t].take_hit(d.normalize_or(Vec2::X) * 2.0, 25);
            }
            rings.push((s.pos, STAR_R, GOLD));
            false
        });
        for (p, r, c) in rings {
            self.ring(p, r, c, 0.35);
        }
        self.fx.retain_mut(|f| {
            f.t -= DT;
            f.t > 0.0
        });
    }

    fn draw_ground(&self) {
        for s in &self.stars {
            let c = proj(s.pos);
            let k = 1.0 - s.t / STAR_T;
            draw_circle(c.x, c.y, STAR_R * S, Color::new(1.0, 0.86, 0.45, 0.08 + 0.2 * k));
            draw_circle_lines(c.x, c.y, STAR_R * S, 2.0, GOLD);
            draw_circle_lines(c.x, c.y, STAR_R * S * k, 2.0, Color::new(1.0, 0.86, 0.45, 0.6));
        }
        for s in &self.shots {
            let c = proj(s.pos);
            draw_circle(c.x, c.y, s.r * S * 0.8, Color::new(0.0, 0.0, 0.0, 0.3));
        }
    }

    fn draw_air(&self) {
        for s in &self.shots {
            let c = proj(s.pos) - vec2(0.0, 9.0 * S);
            let (core, glow) = if s.big { (WHITE, LAV) } else { (WHITE, MINT) };
            let pulse = 1.0 + (get_time() as f32 * 12.0).sin() * 0.1;
            draw_circle(c.x, c.y, s.r * S * 1.6 * pulse, Color::new(glow.r, glow.g, glow.b, 0.3));
            draw_circle(c.x, c.y, s.r * S, glow);
            draw_circle(c.x, c.y, s.r * S * 0.45, core);
        }
        for s in &self.stars {
            let c = proj(s.pos);
            let h = s.t / STAR_T * 80.0 * S;
            let rot = get_time() as f32 * 360.0;
            draw_poly(c.x, c.y - h, 5, 6.0 * S, rot, GOLD);
            draw_poly(c.x, c.y - h, 5, 3.0 * S, rot, WHITE);
        }
        for f in &self.fx {
            let k = f.t / f.max;
            let c = proj(f.pos);
            match f.kind {
                FxKind::Ring(r, col) => {
                    let rr = r * S * (1.2 - k * 0.6);
                    draw_circle_lines(c.x, c.y - 4.0 * S, rr, 3.0, Color::new(col.r, col.g, col.b, k));
                }
                FxKind::Trail(to) => {
                    let e = proj(to);
                    draw_line(c.x, c.y - 9.0 * S, e.x, e.y - 9.0 * S, 6.0 * k, Color::new(0.85, 0.62, 0.95, k));
                }
            }
        }
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

    let total: u32 = anims.get(p.state).frames.iter().map(|f| f.ticks).sum();
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
        let fill = w * p.hp.max(0) as f32 / 100.0;
        let fx = if i == 0 { x } else { x + w - fill };
        draw_rectangle(fx, 24.0, fill, 14.0, p.color);
        for (k, name) in ["LMB", "Q", "E", "R"].iter().enumerate() {
            let bx = if i == 0 { x + k as f32 * 52.0 } else { x + w - 48.0 - k as f32 * 52.0 };
            draw_rectangle(bx, 44.0, 48.0, 22.0, Color::from_rgba(34, 38, 52, 255));
            let frac = p.cd[k] / COOLDOWN[k];
            let col = if frac <= 0.0 { MINT } else { Color::from_rgba(70, 76, 98, 255) };
            draw_rectangle(bx, 44.0, 48.0 * (1.0 - frac), 22.0, col);
            draw_text(name, bx + 6.0, 61.0, 20.0, Color::from_rgba(20, 20, 28, 255));
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let anims = Anims::new();
    let sprites = Sprites::load("assets/witch").await;
    let mut players = [
        Player::new(vec2(120.0, 200.0), 0.0, SKYBLUE, false),
        Player::new(vec2(280.0, 200.0), std::f32::consts::PI, ORANGE, true),
    ];
    let mut pending = Input::default();
    let mut acc = 0.0;
    let mut world = World::default();
    let mut debug = true;

    loop {
        if is_key_pressed(KeyCode::Tab) {
            debug = !debug;
        }
        let r = read_input(mouse_world());
        pending.dir = r.dir;
        pending.aim = r.aim;
        pending.skill = r.skill.or(pending.skill);
        pending.dodge |= r.dodge;

        acc += get_frame_time().min(0.25);
        while acc >= DT {
            for i in 0..2 {
                let input = if players[i].dummy { Input::default() } else { pending };
                players[i].step(input, &anims);
                world.cast(&mut players[i], i);
            }
            pending.skill = None;
            pending.dodge = false;
            resolve_push(&mut players);
            world.update(&mut players, &anims);
            resolve_hits(&mut players, &anims);
            acc -= DT;
        }

        update_camera(&players);
        clear_background(Color::from_rgba(13, 15, 22, 255));
        draw_floor();
        draw_cage(false);
        world.draw_ground();
        let mut order = [0, 1];
        order.sort_by(|a, b| players[*a].pos.y.total_cmp(&players[*b].pos.y));
        for i in order {
            draw_player(&players[i], &anims, &sprites, debug);
        }
        world.draw_air();
        draw_cage(true);
        let me = proj(players[0].pos);
        let aim = proj(mouse_world());
        if is_key_down(KeyCode::R) {
            draw_circle_lines(me.x, me.y, STAR_RANGE * S, 1.5, Color::new(1.0, 0.86, 0.45, 0.35));
        }
        draw_line(me.x, me.y - 9.0 * S, aim.x, aim.y - 9.0 * S, 1.0, Color::new(1.0, 1.0, 1.0, 0.15));
        let m = Vec2::from(mouse_position());
        draw_circle_lines(m.x, m.y, 8.0, 1.5, WHITE);
        draw_circle(m.x, m.y, 2.0, WHITE);
        draw_hud(&players);
        draw_text(
            "WASD move   Mouse aim   LMB bolt   RMB roll   Q orb   E blink   R star   Tab boxes",
            40.0,
            screen_height() - 20.0,
            24.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
