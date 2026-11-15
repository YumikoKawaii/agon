#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn agon_fetch(mp: *const u8, ml: usize, up: *const u8, ul: usize, bp: *const u8, bl: usize, tp: *const u8, tl: usize) -> i32;
    fn agon_status(id: i32) -> i32;
    fn agon_len(id: i32) -> usize;
    fn agon_copy(id: i32, p: *mut u8);
    fn agon_watch(up: *const u8, ul: usize, bp: *const u8, bl: usize, tp: *const u8, tl: usize) -> i32;
    fn agon_watch_next(id: i32) -> i32;
    fn agon_watch_copy(id: i32, p: *mut u8);
    fn agon_watch_close(id: i32);
    fn agon_wt_open(up: *const u8, ul: usize, hp: *const u8, hl: usize) -> i32;
    fn agon_wt_state(id: i32) -> i32;
    fn agon_wt_send(id: i32, p: *const u8, l: usize);
    fn agon_wt_recv(id: i32, p: *mut u8, max: usize) -> usize;
    fn agon_wt_line(id: i32) -> i32;
    fn agon_wt_line_copy(id: i32, p: *mut u8);
    fn agon_wt_close(id: i32);
}

pub struct Req {
    #[cfg(target_arch = "wasm32")]
    id: i32,
    #[cfg(not(target_arch = "wasm32"))]
    done: Option<(i32, String)>,
}

#[cfg(not(target_arch = "wasm32"))]
const OFFLINE: &str = "{\"code\":\"unavailable\",\"message\":\"online features need the web build\"}";

pub fn call(rpc: &str, body: &str, token: &str) -> Req {
    let url = format!("/agon.v1.{rpc}");
    #[cfg(target_arch = "wasm32")]
    {
        let m = "POST";
        let id = unsafe { agon_fetch(m.as_ptr(), m.len(), url.as_ptr(), url.len(), body.as_ptr(), body.len(), token.as_ptr(), token.len()) };
        Req { id }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, body, token);
        Req { done: Some((-1, OFFLINE.to_string())) }
    }
}

impl Req {
    pub fn poll(&mut self) -> Option<(i32, String)> {
        #[cfg(target_arch = "wasm32")]
        {
            let status = unsafe { agon_status(self.id) };
            if status == 0 {
                return None;
            }
            let mut buf = vec![0u8; unsafe { agon_len(self.id) }];
            unsafe { agon_copy(self.id, buf.as_mut_ptr()) };
            Some((status, String::from_utf8_lossy(&buf).into_owned()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.done.take()
        }
    }
}

pub enum Next {
    Msg(String),
    Wait,
    Closed,
}

pub struct Watch {
    #[cfg(target_arch = "wasm32")]
    id: i32,
}

pub fn watch(rpc: &str, body: &str, token: &str) -> Watch {
    let url = format!("/agon.v1.{rpc}");
    #[cfg(target_arch = "wasm32")]
    {
        let id = unsafe { agon_watch(url.as_ptr(), url.len(), body.as_ptr(), body.len(), token.as_ptr(), token.len()) };
        Watch { id }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, body, token);
        Watch {}
    }
}

impl Watch {
    pub fn next(&mut self) -> Next {
        #[cfg(target_arch = "wasm32")]
        {
            let n = unsafe { agon_watch_next(self.id) };
            if n == -1 {
                return Next::Wait;
            }
            if n < 0 {
                return Next::Closed;
            }
            let mut buf = vec![0u8; n as usize];
            unsafe { agon_watch_copy(self.id, buf.as_mut_ptr()) };
            Next::Msg(String::from_utf8_lossy(&buf).into_owned())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Next::Closed
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_watch_close(self.id)
        };
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Link {
    Connecting,
    Open,
    Closed,
}

pub struct Wt {
    #[cfg(target_arch = "wasm32")]
    id: i32,
}

impl Wt {
    pub fn open(url: &str, hash: &str) -> Wt {
        #[cfg(target_arch = "wasm32")]
        {
            let id = unsafe { agon_wt_open(url.as_ptr(), url.len(), hash.as_ptr(), hash.len()) };
            Wt { id }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (url, hash);
            Wt {}
        }
    }

    pub fn state(&self) -> Link {
        #[cfg(target_arch = "wasm32")]
        {
            match unsafe { agon_wt_state(self.id) } {
                0 => Link::Connecting,
                1 => Link::Open,
                _ => Link::Closed,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Link::Closed
        }
    }

    pub fn send(&self, b: &[u8]) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_wt_send(self.id, b.as_ptr(), b.len())
        };
        #[cfg(not(target_arch = "wasm32"))]
        let _ = b;
    }

    pub fn recv(&self, buf: &mut [u8]) -> usize {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { agon_wt_recv(self.id, buf.as_mut_ptr(), buf.len()) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = buf;
            0
        }
    }

    pub fn line(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let n = unsafe { agon_wt_line(self.id) };
            if n < 0 {
                return None;
            }
            let mut buf = vec![0u8; n as usize];
            unsafe { agon_wt_line_copy(self.id, buf.as_mut_ptr()) };
            Some(String::from_utf8_lossy(&buf).into_owned())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }
}

impl Drop for Wt {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_wt_close(self.id)
        };
    }
}

pub const PACKET: usize = 44;

#[derive(Clone, Copy, Default)]
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
    pub fn encode(&self) -> [u8; PACKET] {
        let mut b = [0u8; PACKET];
        let f = [self.dir[0], self.dir[1], self.aim[0], self.aim[1], self.pos[0], self.pos[1], self.ang];
        b[0..4].copy_from_slice(&self.tick.to_le_bytes());
        for (i, v) in f.iter().enumerate() {
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
        if b.len() != PACKET {
            return None;
        }
        let u = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
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
