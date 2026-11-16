use macroquad::prelude::*;

use crate::{
    assets,
    core::{
        magic::{JUDGE_R, JUDGE_RANGE, NEEDLE_R, RAY_CHARGE, RAY_LEN, RAY_SHOW, RAY_W, WARD_OFF, WARD_T},
        hex::{BOLT_R, CAULDRON_R, CAULDRON_RANGE, POTION_R, POTION_RANGE},
        sword::{Formation, INTENT_R, INTENT_RANGE, FORM_APPEAR, FORM_HOVER, FORM_N, FORM_R, FORM_RANGE, FORM_STAB, LUNGE_REACH},
        effect::EffectKind,
        Battle, Skill,
    },
    screen::battle::view::{alpha, glow, magic_circle, Camera, SPRITE, AQUA, CHEST, FOOT, GOLD, ICE, LILAC, MINT, PALE, S},
};

const AURA_BACK_FEET: Vec2 = vec2(25.0, 43.0);
const AURA_FRONT_FEET: Vec2 = vec2(26.0, 63.0);

fn draw_sword(tex: &Texture2D, at: Vec2, ang: f32, scale: f32, a: f32) {
    let size = vec2(tex.width(), tex.height()) * 0.5 * S * scale;
    draw_texture_ex(tex, at.x - size.x / 2.0, at.y - size.y / 2.0, Color::new(1.0, 1.0, 1.0, a.clamp(0.0, 1.0)), DrawTextureParams { dest_size: Some(size), rotation: ang, ..Default::default() });
}

pub fn draw_ground(b: &Battle, cam: &Camera) {
    let time = get_time() as f32;
    for p in b.players.iter().filter(|p| p.intent_ready()) {
        let sh = &assets::get().sword.aura_back;
        sh.draw_at(sh.frame_at(time, true).unwrap_or(0), cam.proj(p.pos), AURA_BACK_FEET, SPRITE * S, 1.0);
    }
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
    for c in &b.hexes.cauldrons {
        let k = c.charge();
        let g = cam.proj(c.pos);
        let r = CAULDRON_R * S * (0.3 + 0.7 * (k * 3.0).min(1.0));
        let sig = &assets::get().witch.sigil;
        sig.draw(sig.frame_at(time, true).unwrap_or(0), g, r / (CAULDRON_R * S) * 0.8 * S, 0.6 + 0.4 * k);
        let bub = (k * 8.0 * std::f32::consts::TAU).sin().abs();
        draw_circle_lines(g.x, g.y, r * (1.0 - k), 2.0, alpha(WHITE, 0.3 + 0.4 * bub));
    }
    let burn = &assets::get().status[EffectKind::Burn.i()];
    for z in &b.zones {
        let g = cam.proj(z.pos);
        let k = z.fade().min(z.t * 4.0).min(1.0);
        let r = z.r * S;
        draw_circle(g.x, g.y, r, alpha(LILAC, 0.18 * k));
        magic_circle(g, r, time * 1.5, MINT, 0.5 * k, 2.0);
        let frame = burn.sheet.frame_at(time, true).unwrap_or(0);
        for n in 0..5 {
            let off = if n == 0 { Vec2::ZERO } else { Vec2::from_angle(n as f32 * 1.57 + 0.4) * z.r * 0.55 };
            burn.sheet.draw_at((frame + n * 3) % burn.sheet.frames, cam.proj(z.pos + off), burn.anchor, 0.35 * S, 0.85 * k);
        }
    }
    for p in &b.hexes.potions {
        let g = cam.proj(p.to);
        let r = POTION_R * S;
        draw_circle(g.x, g.y, r, alpha(LILAC, 0.06 + 0.14 * p.k()));
        draw_circle_lines(g.x, g.y, r, 1.5, alpha(LILAC, 0.5 + 0.4 * p.k()));
        let s = cam.proj(p.from.lerp(p.to, p.k()));
        draw_circle(s.x, s.y, 3.0 * S, Color::new(0.0, 0.0, 0.0, 0.3));
    }
    for bo in &b.hexes.bolts {
        let c = cam.proj(bo.pos);
        draw_circle(c.x, c.y, BOLT_R * S * 0.8, Color::new(0.0, 0.0, 0.0, 0.3));
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
    let fx = &assets::get().sword;
    for p in b.players.iter().filter(|p| p.intent_ready()) {
        let sh = &fx.aura_front;
        sh.draw_at(sh.frame_at(time, true).unwrap_or(0), cam.proj(p.pos), AURA_FRONT_FEET, SPRITE * S, 1.0);
    }
    for w in &b.blades.waves {
        let k = (w.trav / INTENT_RANGE).clamp(0.0, 1.0);
        let i = 2 + (k * 10.0) as usize;
        let pivot = vec2(22.0 + (i - 2) as f32 * 7.0, 24.5);
        let c = cam.air(w.pos, CHEST);
        glow(c, 8.0 * S, AQUA, 0.25 * (1.0 - k * 0.5));
        fx.wave.draw_rot(i, c, pivot, INTENT_R / 22.0 * S, w.ang(), 1.0);
    }
    for s in &b.spells.shots {
        let c = cam.air(s.pos, CHEST);
        let tail = cam.air(s.pos - s.vel.normalize_or_zero() * NEEDLE_R * 4.0, CHEST);
        draw_line(tail.x, tail.y, c.x, c.y, NEEDLE_R * S * 1.2, alpha(ICE, 0.5));
        glow(c, NEEDLE_R * S, ICE, 1.0);
    }
    for bo in &b.hexes.bolts {
        let c = cam.air(bo.pos, CHEST);
        let tail = cam.air(bo.pos - bo.vel.normalize_or_zero() * BOLT_R * 3.0, CHEST);
        draw_line(tail.x, tail.y, c.x, c.y, BOLT_R * S, alpha(LILAC, 0.4));
        let sh = &assets::get().witch.bolt;
        sh.draw(sh.frame_at(time, true).unwrap_or(0), c, 0.5 * S, 1.0);
    }
    for p in &b.hexes.potions {
        let k = p.k();
        let h = CHEST + (k * std::f32::consts::PI).sin() * 50.0;
        let c = cam.air(p.from.lerp(p.to, k), h);
        glow(c, 3.5 * S, LILAC, 1.0);
        draw_circle(c.x, c.y, 2.0 * S, alpha(PALE, 0.9));
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
        let (r, c) = match skills[3] {
            Skill::Judgement => (JUDGE_RANGE, alpha(GOLD, 0.35)),
            Skill::Cauldron => (CAULDRON_RANGE, alpha(LILAC, 0.35)),
            _ => (FORM_RANGE, alpha(AQUA, 0.35)),
        };
        draw_circle_lines(at.x, at.y, r * S, 1.5, c);
    }
    if skills[1] == Skill::Potion && is_key_down(KeyCode::Q) {
        draw_circle_lines(at.x, at.y, POTION_RANGE * S, 1.5, alpha(LILAC, 0.35));
    }
    if skills[1] == Skill::Ray && is_key_down(KeyCode::Q) {
        let d = (mouse - me.pos).normalize_or(Vec2::X);
        let e = cam.proj(me.pos + d * RAY_LEN);
        draw_line(at.x, at.y - FOOT * S, e.x, e.y - FOOT * S, 2.0, alpha(ICE, 0.3));
    }
    let aim = cam.proj(mouse);
    draw_line(at.x, at.y - FOOT * S, aim.x, aim.y - FOOT * S, 1.0, alpha(WHITE, 0.15));
}
