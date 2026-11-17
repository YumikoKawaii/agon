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
    core::parasol::{FLIGHT_RANGE, LEASH, STORM_R, STORM_RANGE, TALISMAN_R},
    screen::battle::view::SAKURA,
};

const AURA_BACK_FEET: Vec2 = vec2(25.0, 43.0);
const AURA_FRONT_FEET: Vec2 = vec2(26.0, 63.0);

fn draw_sword(tex: &Texture2D, at: Vec2, ang: f32, scale: f32, a: f32) {
    let size = vec2(tex.width(), tex.height()) * 0.5 * S * scale;
    draw_texture_ex(tex, at.x - size.x / 2.0, at.y - size.y / 2.0, Color::new(1.0, 1.0, 1.0, a.clamp(0.0, 1.0)), DrawTextureParams { dest_size: Some(size), rotation: ang, ..Default::default() });
}

pub fn draw_ground(b: &Battle, cam: &Camera) {
    draw_wells(b, cam);
    let time = get_time() as f32;
    for (seat, at) in b.parasols.planted.iter().enumerate() {
        if let Some(at) = at {
            let g = cam.proj(*at);
            draw_circle(g.x, g.y, 5.0 * S, Color::new(0.0, 0.0, 0.0, 0.3));
            let near = b.players[seat].pos.distance(*at) < 30.0;
            let far = (b.players[seat].pos.distance(*at) / LEASH - 0.7) / 0.3;
            if far > 0.0 {
                let me = cam.proj(b.players[seat].pos);
                draw_line(me.x, me.y, g.x, g.y, 1.5, alpha(SAKURA, 0.15 + 0.5 * far.min(1.0)));
            }
            magic_circle(g, 9.0 * S, time * 1.5, SAKURA, if near { 0.8 } else { 0.35 }, 1.5);
            glow(g, 8.0 * S, SAKURA, 0.15);
        }
    }
    for s in &b.parasols.storms {
        let k = s.k();
        let g = cam.proj(s.pos);
        let r = STORM_R * S;
        let sheet = &assets::get().parasol.tempest;
        sheet.draw_at(((k * 16.0) as usize).min(15), g, vec2(76.0, 65.0), S * STORM_R / 52.0, 1.0);
        glow(g, r * 0.6, SAKURA, 0.1 + 0.2 * k);
    }
    for fl in &b.parasols.flights {
        let g = cam.proj(fl.pos);
        draw_circle(g.x, g.y, 4.0 * S, Color::new(0.0, 0.0, 0.0, 0.25));
    }
    for t in &b.parasols.talismans {
        let c = cam.proj(t.pos);
        draw_circle(c.x, c.y, TALISMAN_R * S * 0.7, Color::new(0.0, 0.0, 0.0, 0.3));
    }
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
    draw_judge_rune(b, cam, time);
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
    for o in &b.foxes.orbs {
        let c = cam.proj(o.pos);
        draw_circle(c.x, c.y, o.kind.r() * S * 0.8, Color::new(0.0, 0.0, 0.0, 0.3));
    }
    for s in &b.foxes.shines {
        let g = cam.proj(s.pos);
        let sh = &assets::get().fox.sigil;
        let k = crate::core::fox::SHINE_R / 56.0;
        sh.draw_at(sh.frame_at(time, true).unwrap_or(0), g, vec2(72.0, 37.0), k * S, 0.5 + 0.5 * s.k());
    }
}

pub fn draw_air(b: &Battle, cam: &Camera, sword: &Texture2D) {
    let time = get_time() as f32;
    for (seat, at) in b.parasols.planted.iter().enumerate() {
        let Some(at) = at else { continue };
        match b.parasols.storms.iter().find(|s| s.owner == seat) {
            Some(s) => {
                let k = s.k();
                let c = cam.air(*at, 16.0 + 30.0 * (k * 3.0).min(1.0));
                for n in 0..6 {
                    let a = time * 3.0 + n as f32 * std::f32::consts::TAU / 6.0;
                    let e = cam.air(*at + Vec2::from_angle(a) * STORM_R * 0.9, 0.0);
                    draw_line(c.x, c.y, e.x, e.y, 1.5 * S, alpha(SAKURA, 0.25 + 0.3 * k));
                }
                glow(c, 14.0 * S, SAKURA, 0.5 + 0.4 * k);
                draw_parasol_scaled(c, time * (4.0 + 20.0 * k), 1.0, 1.6 + 0.6 * k);
            }
            None => {
                let bob = (time * 2.5).sin() * 1.0;
                draw_parasol(cam.air(*at, 16.0 + bob), time * 0.6, 1.0);
            }
        }
    }
    for fl in &b.parasols.flights {
        draw_parasol(cam.air(fl.pos, CHEST + 6.0), fl.spin, 1.0);
    }
    for t in &b.parasols.talismans {
        let c = cam.air(t.pos, CHEST);
        let d = t.vel.normalize_or(Vec2::X);
        let tail = cam.air(t.pos - d * TALISMAN_R * 3.0, CHEST);
        draw_line(tail.x, tail.y, c.x, c.y, TALISMAN_R * S, alpha(SAKURA, 0.35));
        glow(c, TALISMAN_R * S, SAKURA, 0.6);
        let bolt = &assets::get().parasol.bolt;
        bolt.draw_rot(bolt.frame_at(t.age, true).unwrap_or(0), c, vec2(34.0, 12.0), 0.7 * S, d.y.atan2(d.x), 1.0);
    }
    for o in &b.foxes.orbs {
        let c = cam.air(o.pos, CHEST);
        let d = o.vel.normalize_or(Vec2::X);
        let r = o.kind.r();
        let fx = &assets::get().fox;
        let (sh, col) = match o.kind {
            crate::core::fox::OrbKind::Wisp => (&fx.wisp, MINT),
            crate::core::fox::OrbKind::Blind => (&fx.blast, GOLD),
            _ => (&fx.orb, MINT),
        };
        let ang = d.y.atan2(d.x);
        match o.kind {
            crate::core::fox::OrbKind::Blind => {
                glow(c, r * S * 2.0, col, 0.5);
                sh.draw_at(0, c, vec2(80.0, 47.0), 0.35 * S, 1.0);
            }
            crate::core::fox::OrbKind::Wisp => sh.draw_rot(sh.frame_at(o.age, true).unwrap_or(0), c, vec2(54.0, 17.0), r / 8.0 * S, ang, 1.0),
            _ => sh.draw_rot(sh.frame_at(o.age, true).unwrap_or(0), c, vec2(44.0, 19.0), r / 8.0 * S, ang, 1.0),
        }
    }
    draw_seren(b, cam);
    draw_wards(b, cam, time);
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
    draw_needles(b, cam, time);
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
    draw_rays(b, cam, time);
    draw_falling_star(b, cam, time);
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
            let c = cam.air(fm.sword_pos(n), (1.0 - appear) * 30.0 + bob);
            if s > 0.0 {
                let back = cam.proj(fm.center + d * FORM_R);
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
        let (r, c) = match me.skill(3) {
            Skill::Blossom => (crate::core::parasol::BLOSSOM_R, alpha(SAKURA, 0.35)),
            Skill::Judgement => (JUDGE_RANGE, alpha(GOLD, 0.35)),
            Skill::Cauldron => (CAULDRON_RANGE, alpha(LILAC, 0.35)),
            Skill::Storm => (STORM_RANGE, alpha(SAKURA, 0.35)),
            _ => (FORM_RANGE, alpha(AQUA, 0.35)),
        };
        draw_circle_lines(at.x, at.y, r * S, 1.5, c);
    }
    if skills[1] == Skill::Parasol && is_key_down(KeyCode::Q) {
        draw_circle_lines(at.x, at.y, FLIGHT_RANGE * S, 1.5, alpha(SAKURA, 0.35));
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

fn draw_parasol(c: Vec2, spin: f32, a: f32) {
    draw_parasol_scaled(c, spin, a, 1.0);
}

fn draw_parasol_scaled(c: Vec2, spin: f32, a: f32, scale: f32) {
    let r = 8.0 * S * scale;
    glow(c, r * 1.1, SAKURA, 0.2 * a);
    let top = &assets::get().parasol.top;
    top.draw_rot(0, c, vec2(24.0, 24.0), r / 23.0, spin, a);
}

const LEAF: Color = Color::new(0.55, 1.0, 0.82, 1.0);
const MOON: Color = Color::new(0.8, 0.68, 1.0, 1.0);

fn star(c: Vec2, r: f32, inner: f32, n: usize, rot: f32, col: Color, w: f32) {
    let pts: Vec<Vec2> = (0..n * 2)
        .map(|i| c + Vec2::from_angle(rot + i as f32 * std::f32::consts::PI / n as f32) * if i % 2 == 0 { r } else { r * inner })
        .collect();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        draw_line(a.x, a.y, b.x, b.y, w, col);
    }
}

fn sparkle(c: Vec2, r: f32, col: Color) {
    draw_line(c.x - r, c.y, c.x + r, c.y, 1.0, col);
    draw_line(c.x, c.y - r, c.x, c.y + r, 1.0, col);
    draw_circle(c.x, c.y, r * 0.3, alpha(WHITE, col.a));
}

fn leaf(c: Vec2, d: Vec2, len: f32, wid: f32, col: Color) {
    let tip = c + d * len;
    let back = c - d * len * 0.6;
    let l = c + d.perp() * wid;
    let r = c - d.perp() * wid;
    draw_triangle(tip, l, r, col);
    draw_triangle(back, l, r, col);
}

fn draw_needles(b: &Battle, cam: &Camera, time: f32) {
    for s in &b.spells.shots {
        let d = s.vel.normalize_or(Vec2::X);
        let c = cam.air(s.pos, CHEST);
        let sd = (cam.air(s.pos + d, CHEST) - c).normalize_or(Vec2::X);
        for n in 1..5 {
            let k = n as f32 / 5.0;
            let t = cam.air(s.pos - d * NEEDLE_R * 3.0 * n as f32, CHEST + (time * 20.0 + n as f32 * 1.7).sin() * 1.5);
            sparkle(t, (1.0 - k) * 2.0 * S, alpha(if n % 2 == 0 { MOON } else { LEAF }, 0.9 - k * 0.7));
        }
        glow(c, NEEDLE_R * S * 1.2, LEAF, 0.35);
        leaf(c, sd, NEEDLE_R * S * 2.4, NEEDLE_R * S * 0.9, LEAF);
        leaf(c, sd, NEEDLE_R * S * 1.6, NEEDLE_R * S * 0.4, PALE);
    }
}

fn rune_circle(c: Vec2, r: f32, rot: f32, col: Color, a: f32) {
    draw_circle_lines(c.x, c.y, r, 2.0, alpha(col, a));
    draw_circle_lines(c.x, c.y, r * 0.78, 1.0, alpha(col, a * 0.8));
    draw_circle_lines(c.x, c.y, r * 0.42, 1.5, alpha(col, a * 0.9));
    for n in 0..12 {
        let ang = rot + n as f32 * std::f32::consts::TAU / 12.0;
        let p = c + Vec2::from_angle(ang) * r * 0.89;
        let t = Vec2::from_angle(ang).perp() * r * 0.06;
        draw_line(p.x - t.x, p.y - t.y, p.x + t.x, p.y + t.y, 1.5, alpha(PALE, a));
    }
    star(c, r * 0.76, 0.55, 6, -rot * 1.3, alpha(col, a * 0.7), 1.0);
    star(c, r * 0.42, 0.3, 4, rot * 2.0, alpha(WHITE, a * 0.8), 1.0);
}

fn draw_seren(b: &Battle, cam: &Camera) {
    let fx = &assets::get().seren;
    let r = crate::core::seren::SHOT_R;
    for s in &b.serens.shots {
        let c = cam.air(s.pos, CHEST);
        let d = s.vel.normalize_or(Vec2::X);
        let tail = cam.air(s.pos - d * r * 3.0, CHEST);
        draw_line(tail.x, tail.y, c.x, c.y, r * S, alpha(ICE, 0.35));
        glow(c, r * S * 1.6, ICE, 0.5);
        fx.shot.draw(fx.shot.frame_at(s.age, true).unwrap_or(0), c, r / 9.0 * S, 1.0);
    }
    for l in &b.serens.lances {
        let k = 1.0 - l.t / crate::core::seren::LANCE_SHOW;
        let c = cam.air(l.origin, CHEST);
        let e = cam.air(l.end(), CHEST);
        let col = if l.big { GOLD } else { ICE };
        draw_line(c.x, c.y, e.x, e.y, l.width() * S * 1.6 * k, alpha(col, 0.35 * k));
        draw_line(c.x, c.y, e.x, e.y, l.width() * S * 0.6 * k, alpha(WHITE, 0.95 * k));
        for n in 1..6 {
            let p = c.lerp(e, n as f32 / 6.0);
            let rr = (2.0 + 2.0 * (1.0 - k)) * S;
            draw_circle_lines(p.x, p.y, rr, 1.0, alpha(GOLD, 0.8 * k));
        }
        glow(c, 6.0 * S, col, 0.7 * k);
    }
    for w in &b.serens.waves {
        let c = cam.air(w.pos, CHEST);
        let i = ((w.dist / w.range()) * 5.0) as usize + 1;
        let scale = if w.big { 0.75 } else { 0.45 };
        if w.big {
            glow(c, 16.0 * S, GOLD, 0.35);
        }
        fx.wave.draw_rot(i.min(5), c, vec2(48.0, 48.0), scale * S, w.dir.y.atan2(w.dir.x), 1.0);
        glow(c, 8.0 * S, ICE, 0.35);
    }
    for bm in &b.serens.beams {
        let i = ((bm.t * fx.beam.fps) as usize).min(fx.beam.frames - 1);
        fx.beam.draw_rot(i, cam.air(bm.origin, CHEST), vec2(40.0, 42.0), crate::core::seren::BEAM_RANGE / 228.0 * S, bm.dir.y.atan2(bm.dir.x), 1.0);
        if bm.over {
            glow(cam.air(bm.origin, CHEST), (6.0 + 6.0 * bm.charge()) * S, GOLD, 0.35);
        }
    }
}

fn draw_wards(b: &Battle, cam: &Camera, time: f32) {
    for w in b.spells.wards.iter() {
        let p = &b.players[w.owner];
        let blink = w.t > WARD_T - 0.4 && (w.t * 30.0) as i32 % 2 == 0;
        let open = (w.t / 0.12).min(1.0);
        let fade = open * if blink { 0.3 } else { 1.0 };
        let f = p.facing();
        let center = p.pos + f * WARD_OFF;
        let sweep = ((w.t * 1.6).fract() * 2.0 - 1.0) * 2.0;
        let rows: [(f32, &[f32]); 2] = [(0.0, &[-2.0, -1.0, 0.0, 1.0, 2.0]), (-7.0, &[-1.5, -0.5, 0.5, 1.5])];
        glow(cam.air(center, CHEST), 14.0 * S * open, ICE, 0.12 * fade);
        for (back, cols) in rows {
            for &i in cols {
                let spread = i * 9.5 * open;
                let world = center + f * back + f.perp() * spread - f * (i.abs() * 2.0);
                let cell = cam.air(world, CHEST + if back < 0.0 { 5.0 } else { 0.0 });
                let lit = (1.0 - (i - sweep).abs()).max(0.0);
                let pulse = 0.7 + 0.3 * (time * 7.0 + i).sin();
                let rr = 5.6 * S * (0.6 + 0.4 * open);
                let a = if back < 0.0 { 0.55 } else { 1.0 } * fade;
                draw_poly(cell.x, cell.y, 6, rr, 30.0, alpha(ICE, (0.16 * pulse + 0.35 * lit) * a));
                draw_poly_lines(cell.x, cell.y, 6, rr, 30.0, 1.8, alpha(PALE, 0.9 * a));
                draw_poly_lines(cell.x, cell.y, 6, rr * 0.55, 30.0 + time * 120.0, 1.0, alpha(WHITE, (0.4 + 0.6 * lit) * a));
                if lit > 0.6 {
                    sparkle(cell, 2.5 * S, alpha(WHITE, lit * a));
                }
            }
        }
        let foot = cam.proj(p.pos);
        draw_circle_lines(foot.x, foot.y, 12.0 * S * open, 1.5, alpha(ICE, 0.5 * fade));
    }
}

fn draw_rays(b: &Battle, cam: &Camera, time: f32) {
    for r in &b.spells.rays {
        let a = cam.air(r.tip(), CHEST);
        let e = cam.air(r.end(), CHEST);
        let d = (e - a).normalize_or(Vec2::X);
        if !r.fired {
            let k = r.t / RAY_CHARGE;
            let flick = if (time * 40.0) as i32 % 2 == 0 { 1.0 } else { 0.6 };
            draw_line(a.x, a.y, e.x, e.y, 1.0 + k * 2.0, alpha(ICE, 0.25 * flick));
            rune_circle(a, (7.0 + 7.0 * k) * S, time * 5.0, ICE, 0.95);
            rune_circle(a + d * 7.0 * S, (4.0 + 4.0 * k) * S, -time * 8.0, PALE, 0.5 + 0.5 * k);
            for n in 0..10 {
                let ang = time * 11.0 + n as f32 * std::f32::consts::TAU / 10.0;
                let m = a + Vec2::from_angle(ang) * (1.0 - k) * 22.0 * S;
                draw_line(m.x, m.y, a.x, a.y, 1.0, alpha(PALE, 0.25 * k));
                glow(m, 1.0 * S, ICE, 0.8);
            }
            glow(a, (2.0 + 4.0 * k) * S, PALE, 0.95);
        } else {
            let k = 1.0 - (r.t - RAY_CHARGE) / RAY_SHOW;
            let w = RAY_W * S * (0.3 + 1.0 * k) * macroquad::rand::gen_range(0.88, 1.12);
            for (mul, col, op) in [(3.4, ICE, 0.1), (2.4, ICE, 0.22), (1.5, ICE, 0.55), (0.75, PALE, 1.0), (0.32, WHITE, 1.0)] {
                draw_line(a.x, a.y, e.x, e.y, w * mul, alpha(col, op * k));
            }
            let len = a.distance(e);
            for n in 0..4 {
                let t = ((time * 3.0 + n as f32 * 0.25).fract()) * len;
                let c = a + d * t;
                let rr = w * 1.3;
                draw_line(c.x - d.perp().x * rr, c.y - d.perp().y * rr, c.x + d.perp().x * rr, c.y + d.perp().y * rr, 2.0, alpha(WHITE, 0.6 * k));
            }
            rune_circle(a, w * 2.2, time * 9.0, ICE, k);
            glow(a, w * 1.1, WHITE, k);
            glow(e, w * 0.7, ICE, k);
        }
    }
}

fn heaven_col(n: usize) -> Color {
    match n % 3 {
        0 => GOLD,
        1 => PALE,
        _ => ICE,
    }
}

fn draw_judge_rune(b: &Battle, cam: &Camera, time: f32) {
    use crate::core::magic::{JUDGE_T, HEAVEN_SHOW};
    for j in &b.spells.judges {
        let c = cam.proj(j.pos);
        if j.t < JUDGE_T {
            let k = j.t / JUDGE_T;
            let open = (k * 4.0).min(1.0);
            let r = JUDGE_R * S * (0.4 + 0.6 * open);
            draw_circle(c.x, c.y, r, alpha(GOLD, 0.05 + 0.15 * k));
            rune_circle(c, r, time * (0.8 + 4.0 * k), GOLD, 0.5 + 0.5 * k);
            star(c, r * 0.95, 0.5, 8, -time * (0.5 + 2.0 * k), alpha(PALE, 0.3 + 0.5 * k), 1.5);
            draw_circle(c.x, c.y, 14.0 * S, alpha(WHITE, 0.08 + 0.25 * k));
            draw_circle_lines(c.x, c.y, 14.0 * S, 2.0, alpha(WHITE, 0.4 + 0.6 * k));
            for n in 0..3 {
                let p = ((k * 3.0 + n as f32 / 3.0) % 1.0).max(0.0);
                draw_circle_lines(c.x, c.y, r * (1.0 - p), 1.5, alpha(WHITE, 0.5 * p));
            }
            let me = cam.proj(j.from);
            rune_circle(me, 16.0 * S * open, -time * 3.0, GOLD, 0.6 * open);
        } else {
            let k = 1.0 - (j.t - JUDGE_T) / HEAVEN_SHOW;
            let r = JUDGE_R * S * (1.0 + 0.4 * (1.0 - k));
            draw_circle(c.x, c.y, r, alpha(WHITE, 0.35 * k * k));
            rune_circle(c, r, time * 8.0, GOLD, k);
            star(c, r * (1.2 - 0.3 * k), 0.25, 8, time * 3.0, alpha(WHITE, k), 2.5);
        }
    }
}

fn draw_falling_star(b: &Battle, cam: &Camera, time: f32) {
    use crate::core::magic::{HEAVEN_H, HEAVEN_N, HEAVEN_SHOW, JUDGE_T};
    for j in &b.spells.judges {
        let target = cam.air(j.pos, CHEST);
        let crown = cam.air(j.from, HEAVEN_H + 14.0);
        if j.t < JUDGE_T {
            let k = j.t / JUDGE_T;
            rune_circle(crown, (8.0 + 6.0 * k) * S, time * 4.0, GOLD, 0.4 + 0.6 * k);
            glow(crown, (2.0 + 3.0 * k) * S, WHITE, 0.8);
            for n in 0..HEAVEN_N {
                let appear = ((j.t - n as f32 * 0.025) / 0.15).clamp(0.0, 1.0);
                if appear <= 0.0 {
                    continue;
                }
                let col = heaven_col(n);
                let bob = (time * 4.0 + n as f32).sin() * 2.0;
                let e = cam.air(j.emitter(n), HEAVEN_H + bob);
                let flick = if ((time * 30.0) as usize + n) % 2 == 0 { 1.0 } else { 0.55 };
                draw_line(e.x, e.y, target.x, target.y, 0.8 + 1.5 * k, alpha(col, (0.08 + 0.35 * k * k) * flick * appear));
                draw_line(e.x, e.y, crown.x, crown.y, 1.0, alpha(GOLD, 0.3 * appear));
                rune_circle(e, (3.5 + 2.5 * k) * S * appear, time * (3.0 + 8.0 * k) * if n % 2 == 0 { 1.0 } else { -1.0 }, col, appear);
                glow(e, (1.0 + 2.2 * k) * S * appear, col, 0.7 + 0.3 * k);
            }
            glow(target, (2.0 + 6.0 * k) * S, GOLD, 0.3 + 0.6 * k);
        } else {
            let k = 1.0 - (j.t - JUDGE_T) / HEAVEN_SHOW;
            let w = 5.0 * S * (0.3 + 0.9 * k) * macroquad::rand::gen_range(0.85, 1.15);
            for n in 0..HEAVEN_N {
                let col = heaven_col(n);
                let e = cam.air(j.emitter(n), HEAVEN_H);
                for (mul, c, op) in [(2.6, col, 0.15), (1.5, col, 0.5), (0.7, PALE, 0.95), (0.3, WHITE, 1.0)] {
                    draw_line(e.x, e.y, target.x, target.y, w * mul, alpha(c, op * k));
                }
                rune_circle(e, 5.0 * S * (1.0 + 0.5 * (1.0 - k)), time * 12.0, col, k);
                glow(e, w * 0.8, WHITE, k);
            }
            let sky = cam.air(j.pos, 220.0);
            for (mul, c, op) in [(4.0, GOLD, 0.15), (2.0, GOLD, 0.45), (0.9, WHITE, 1.0)] {
                draw_line(sky.x, sky.y, target.x, target.y, w * mul * k, alpha(c, op * k));
            }
            glow(target, JUDGE_R * S * 0.5 * (0.6 + 0.4 * k), WHITE, k);
            star(target, JUDGE_R * S * (1.6 - 0.8 * k), 0.18, 4, 0.0, alpha(WHITE, k), 3.0);
            star(target, JUDGE_R * S * (1.2 - 0.5 * k), 0.22, 4, std::f32::consts::FRAC_PI_4, alpha(GOLD, k), 2.0);
        }
    }
}

fn draw_wells(b: &Battle, cam: &Camera) {
    let fx = &assets::get().seren;
    for w in &b.serens.wells {
        let k = (w.t / crate::core::seren::WELL_T).min(1.0);
        let at = cam.air(w.pos, 2.0);
        glow(at, crate::core::seren::WELL_R * S * (1.1 - 0.3 * k), MOON, 0.08 + 0.1 * k);
        let grow = (w.t / 0.15).min(1.0);
        let fade = if w.t > crate::core::seren::WELL_T - 0.2 { 0.3 } else { 0.5 };
        fx.well.draw_at(fx.well.frame_at(w.t, true).unwrap_or(0), at, vec2(64.0, 80.0), crate::core::seren::WELL_R / 58.0 * S * grow, fade);
    }
}
