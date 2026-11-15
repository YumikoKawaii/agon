use macroquad::prelude::*;

use crate::{
    core::{
        magic::{JUDGE_R, JUDGE_RANGE, NEEDLE_R, RAY_CHARGE, RAY_LEN, RAY_SHOW, RAY_W, WARD_OFF, WARD_T},
        sword::{Formation, FORM_APPEAR, FORM_HOVER, FORM_N, FORM_R, FORM_RANGE, FORM_STAB, LUNGE_REACH},
        Battle, Skill,
    },
    screen::battle::view::{alpha, glow, magic_circle, Camera, AQUA, CHEST, FOOT, GOLD, ICE, PALE, S},
};

fn draw_sword(tex: &Texture2D, at: Vec2, ang: f32, scale: f32, a: f32) {
    let size = vec2(tex.width(), tex.height()) * 0.5 * S * scale;
    draw_texture_ex(tex, at.x - size.x / 2.0, at.y - size.y / 2.0, Color::new(1.0, 1.0, 1.0, a.clamp(0.0, 1.0)), DrawTextureParams { dest_size: Some(size), rotation: ang, ..Default::default() });
}

pub fn draw_ground(b: &Battle, cam: &Camera) {
    let time = get_time() as f32;
    for fm in &b.blades.forms {
        let c = cam.proj(fm.center);
        let open = (fm.t / FORM_APPEAR).min(1.0);
        let r = FORM_R * S * (0.4 + 0.6 * open);
        draw_circle(c.x, c.y, r, alpha(AQUA, 0.08 + 0.1 * open));
        magic_circle(c, r, -time * 2.5, AQUA, 0.4 + 0.5 * open, 2.0);
    }
    for j in &b.spells.judges {
        let k = j.charge();
        let open = (k * 3.0).min(1.0);
        let c = cam.proj(j.pos);
        let r = JUDGE_R * S * (0.3 + 0.7 * open);
        draw_circle(c.x, c.y, r, alpha(GOLD, 0.06 + 0.22 * k));
        magic_circle(c, r, time * (1.0 + k * 3.0), GOLD, 0.5 + 0.5 * k, 3.0);
        let pulse = (k * 6.0 * std::f32::consts::TAU).sin().abs();
        draw_circle_lines(c.x, c.y, r * (1.0 - k), 2.0, alpha(WHITE, 0.4 + 0.4 * pulse));
    }
    for s in &b.spells.shots {
        let c = cam.proj(s.pos);
        draw_circle(c.x, c.y, NEEDLE_R * S * 0.8, Color::new(0.0, 0.0, 0.0, 0.3));
    }
}

pub fn draw_air(b: &Battle, cam: &Camera, sword: &Texture2D) {
    let time = get_time() as f32;
    for w in &b.spells.wards {
        let p = &b.players[w.owner];
        let blink = w.t > WARD_T - 0.4 && (w.t * 30.0) as i32 % 2 == 0;
        let fade = (w.t / 0.1).min(1.0) * if blink { 0.3 } else { 1.0 };
        let f = p.facing();
        let center = p.pos + f * WARD_OFF;
        for i in -1..=1 {
            let cell = cam.air(center + f.perp() * i as f32 * 14.5, CHEST);
            let pulse = 0.7 + 0.3 * (time * 6.0 + i as f32).sin();
            draw_poly(cell.x, cell.y, 6, 8.5 * S, 30.0, alpha(ICE, 0.18 * fade * pulse));
            draw_poly_lines(cell.x, cell.y, 6, 8.5 * S, 30.0, 2.0, alpha(PALE, 0.9 * fade));
            draw_poly_lines(cell.x, cell.y, 6, 5.0 * S, 30.0 + time * 90.0, 1.0, alpha(ICE, 0.6 * fade));
        }
    }
    for s in &b.spells.shots {
        let c = cam.air(s.pos, CHEST);
        let tail = cam.air(s.pos - s.vel.normalize_or_zero() * NEEDLE_R * 4.0, CHEST);
        draw_line(tail.x, tail.y, c.x, c.y, NEEDLE_R * S * 1.2, alpha(ICE, 0.5));
        glow(c, NEEDLE_R * S, ICE, 1.0);
    }
    for r in &b.spells.rays {
        let a = cam.air(r.tip(), CHEST);
        let e = cam.air(r.end(), CHEST);
        if !r.fired {
            let k = r.t / RAY_CHARGE;
            let flick = if (time * 40.0) as i32 % 2 == 0 { 1.0 } else { 0.6 };
            draw_line(a.x, a.y, e.x, e.y, 1.0 + k * 2.0, alpha(ICE, 0.25 * flick));
            magic_circle(a, (6.0 + 6.0 * k) * S, time * 6.0, ICE, 0.9, 2.0);
            glow(a, (2.0 + 3.0 * k) * S, PALE, 0.9);
        } else {
            let k = 1.0 - (r.t - RAY_CHARGE) / RAY_SHOW;
            let w = RAY_W * S * (0.3 + 0.9 * k) * macroquad::rand::gen_range(0.85, 1.15);
            for (mul, col, op) in [(2.6, ICE, 0.18), (1.5, ICE, 0.55), (0.7, PALE, 1.0), (0.3, WHITE, 1.0)] {
                draw_line(a.x, a.y, e.x, e.y, w * mul, alpha(col, op * k));
            }
            glow(a, w * 0.9, PALE, k);
            glow(e, w * 0.6, ICE, k);
        }
    }
    for j in &b.spells.judges {
        let k = j.charge();
        let c = cam.air(j.pos, 70.0);
        magic_circle(c, JUDGE_R * S * 0.7 * (k * 2.0).min(1.0), -time * 2.0, PALE, 0.35 + 0.5 * k, 2.0);
        if k > 0.6 {
            let g = cam.proj(j.pos);
            let kk = (k - 0.6) / 0.4;
            draw_line(c.x, c.y, g.x, g.y, 1.0 + kk * 4.0, alpha(PALE, 0.5 * kk));
        }
    }
    for l in &b.blades.lunges {
        let p = &b.players[l.owner];
        let tip = p.pos + l.dir * LUNGE_REACH;
        let a = cam.air(p.pos, CHEST);
        let e = cam.air(tip + l.dir * 6.0, CHEST);
        draw_line(a.x, a.y, e.x, e.y, 7.0 * S, alpha(AQUA, 0.2));
        draw_line(a.x, a.y, e.x, e.y, 2.5 * S, alpha(PALE, 0.7));
        draw_sword(sword, cam.air(tip, CHEST), l.ang(), 1.2, 1.0);
        glow(e, 2.5 * S, AQUA, 0.9);
    }
    for fm in &b.blades.forms {
        let appear = (fm.t / FORM_APPEAR).min(1.0);
        for n in 0..FORM_N {
            let start = Formation::stab_start(n);
            if fm.t > start + FORM_STAB {
                continue;
            }
            let d = fm.dir(n);
            let s = fm.stab(n).clamp(0.0, 1.0);
            let bob = (time * 6.0 + n as f32).sin() * 1.5 * (1.0 - s);
            let c = cam.air(fm.sword_pos(n), CHEST + (1.0 - appear) * 30.0 + bob);
            if s > 0.0 {
                let back = cam.air(fm.center + d * FORM_R, CHEST);
                draw_line(back.x, back.y, c.x, c.y, 4.0 * S, alpha(AQUA, 0.35));
                draw_line(back.x, back.y, c.x, c.y, 1.2 * S, alpha(WHITE, 0.8));
            } else if fm.t > FORM_HOVER - 0.2 {
                let shake = vec2(macroquad::rand::gen_range(-1.0, 1.0), macroquad::rand::gen_range(-1.0, 1.0)) * 2.0;
                glow(c + shake, 3.0 * S, AQUA, 0.5);
            }
            glow(c, 2.0 * S, AQUA, 0.35 * appear);
            draw_sword(sword, c, (-d).y.atan2(-d.x), 1.0, appear);
        }
    }
}

pub fn draw_aim(b: &Battle, cam: &Camera) {
    let me = &b.players[0];
    let at = cam.proj(me.pos);
    let mouse = cam.mouse_world();
    let skills = me.kit().skills;
    if is_key_down(KeyCode::R) {
        let (r, c) = if skills[3] == Skill::Judgement { (JUDGE_RANGE, alpha(GOLD, 0.35)) } else { (FORM_RANGE, alpha(AQUA, 0.35)) };
        draw_circle_lines(at.x, at.y, r * S, 1.5, c);
    }
    if skills[1] == Skill::Ray && is_key_down(KeyCode::Q) {
        let d = (mouse - me.pos).normalize_or(Vec2::X);
        let e = cam.proj(me.pos + d * RAY_LEN);
        draw_line(at.x, at.y - FOOT * S, e.x, e.y - FOOT * S, 2.0, alpha(ICE, 0.3));
    }
    let aim = cam.proj(mouse);
    draw_line(at.x, at.y - FOOT * S, aim.x, aim.y - FOOT * S, 1.0, alpha(WHITE, 0.15));
}
