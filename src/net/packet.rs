pub const PACKET_LEN: usize = 44;

#[derive(Clone, Copy, Default, Debug)]
pub struct Packet {
    pub tick: u32,
    pub dir: [f32; 2],
    pub aim: [f32; 2],
    pub pos: [f32; 2],
    pub ang: f32,
    pub hp: i32,
    pub deaths: u32,
    pub cast_seq: u16,
    pub cast_slot: u8,
    pub dodge_seq: u8,
}

impl Packet {
    pub fn encode(&self) -> [u8; PACKET_LEN] {
        let mut b = [0u8; PACKET_LEN];
        let floats = [self.dir[0], self.dir[1], self.aim[0], self.aim[1], self.pos[0], self.pos[1], self.ang];
        b[0..4].copy_from_slice(&self.tick.to_le_bytes());
        for (i, v) in floats.iter().enumerate() {
            b[4 + i * 4..8 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        b[32..36].copy_from_slice(&self.hp.to_le_bytes());
        b[36..40].copy_from_slice(&self.deaths.to_le_bytes());
        b[40..42].copy_from_slice(&self.cast_seq.to_le_bytes());
        b[42] = self.cast_slot;
        b[43] = self.dodge_seq;
        b
    }

    pub fn decode(b: &[u8]) -> Option<Packet> {
        if b.len() != PACKET_LEN {
            return None;
        }
        let u = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let f = |i: usize| f32::from_bits(u(i));
        Some(Packet {
            tick: u(0),
            dir: [f(4), f(8)],
            aim: [f(12), f(16)],
            pos: [f(20), f(24)],
            ang: f(28),
            hp: u(32) as i32,
            deaths: u(36),
            cast_seq: u16::from_le_bytes([b[40], b[41]]),
            cast_slot: b[42],
            dodge_seq: b[43],
        })
    }
}
