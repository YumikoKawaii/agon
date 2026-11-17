use std::{fs, path::Path};

use macroquad::texture::Image;

const DIRS: [&str; 8] = ["east", "south-east", "south", "south-west", "west", "north-west", "north", "north-east"];

fn frames(dir: &Path) -> Vec<Image> {
    let mut out = Vec::new();
    for i in 0.. {
        let p = dir.join(format!("frame_{i:03}.png"));
        let Ok(bytes) = fs::read(&p) else { break };
        out.push(Image::from_file_with_format(&bytes, None).expect("decode"));
    }
    out
}

fn pack(clip: &Path) {
    let rows: Vec<Vec<Image>> = DIRS.iter().map(|d| frames(&clip.join(d))).collect();
    let cw = rows.iter().flatten().map(|f| f.width).max().unwrap_or(0) as usize;
    let ch = rows.iter().flatten().map(|f| f.height).max().unwrap_or(0) as usize;
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cw == 0 || cols == 0 {
        return;
    }
    let (w, h) = (cw * cols, ch * DIRS.len());
    let mut bytes = vec![0u8; w * h * 4];
    for (r, row) in rows.iter().enumerate() {
        for (c, f) in row.iter().enumerate() {
            let (fw, fh) = (f.width as usize, f.height as usize);
            let ox = c * cw + (cw - fw) / 2;
            let oy = r * ch + (ch - fh) / 2;
            for y in 0..fh {
                let src = &f.bytes[y * fw * 4..(y + 1) * fw * 4];
                let at = ((oy + y) * w + ox) * 4;
                bytes[at..at + fw * 4].copy_from_slice(src);
            }
        }
    }
    let mut flipped = vec![0u8; bytes.len()];
    for y in 0..h {
        flipped[y * w * 4..(y + 1) * w * 4].copy_from_slice(&bytes[(h - 1 - y) * w * 4..(h - y) * w * 4]);
    }
    let img = Image { width: w as u16, height: h as u16, bytes: flipped };
    img.export_png(clip.join("sheet.png").to_str().unwrap());
    let counts: Vec<String> = rows.iter().map(|r| r.len().to_string()).collect();
    fs::write(clip.join("sheet.txt"), format!("{cw} {ch} {}\n", counts.join(" "))).unwrap();
    for d in DIRS {
        let p = clip.join(d);
        if p.is_dir() {
            fs::remove_dir_all(p).unwrap();
        }
    }
    println!("{} {}x{} {}", clip.display(), w, h, counts.join(" "));
}

fn idle(skin: &Path) {
    let dir = skin.join("animations/idle");
    fs::create_dir_all(&dir).unwrap();
    for d in DIRS {
        let bytes = fs::read(skin.join(format!("rotations/{d}.png"))).expect("rotation");
        let base = Image::from_file_with_format(&bytes, None).expect("decode");
        let (w, h) = (base.width as usize, base.height as usize);
        let out = dir.join(d);
        fs::create_dir_all(&out).unwrap();
        for (i, dy) in [0usize, 0, 1, 1].iter().enumerate() {
            let mut px = vec![0u8; w * h * 4];
            for y in 0..h - dy {
                px[(y + dy) * w * 4..(y + dy + 1) * w * 4].copy_from_slice(&base.bytes[y * w * 4..(y + 1) * w * 4]);
            }
            let mut flipped = vec![0u8; px.len()];
            for y in 0..h {
                flipped[y * w * 4..(y + 1) * w * 4].copy_from_slice(&px[(h - 1 - y) * w * 4..(h - y) * w * 4]);
            }
            Image { width: w as u16, height: h as u16, bytes: flipped }.export_png(out.join(format!("frame_{i:03}.png")).to_str().unwrap());
        }
    }
    pack(&dir);
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("idle") {
        idle(Path::new(&std::env::args().nth(2).expect("skin dir")));
        return;
    }
    let root = std::env::args().nth(1).unwrap_or_else(|| "assets".into());
    for skin in fs::read_dir(&root).unwrap().flatten() {
        let anims = skin.path().join("animations");
        let Ok(clips) = fs::read_dir(&anims) else { continue };
        for clip in clips.flatten() {
            if clip.path().join("south").is_dir() || clip.path().join("east").is_dir() {
                pack(&clip.path());
            }
        }
    }
}
