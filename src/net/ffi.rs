#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn agon_fetch(mp: *const u8, ml: usize, up: *const u8, ul: usize, bp: *const u8, bl: usize, tp: *const u8, tl: usize) -> i32;
    fn agon_status(id: i32) -> i32;
    fn agon_len(id: i32) -> usize;
    fn agon_copy(id: i32, p: *mut u8);
    fn agon_drop(id: i32);
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

#[cfg(not(target_arch = "wasm32"))]
const OFFLINE: &str = "{\"code\":\"unavailable\",\"message\":\"online features need the web build\"}";

pub struct Fetch {
    #[cfg(target_arch = "wasm32")]
    id: i32,
    #[cfg(not(target_arch = "wasm32"))]
    done: Option<(i32, String)>,
}

impl Fetch {
    pub fn post(url: &str, body: &str, token: &str) -> Fetch {
        #[cfg(target_arch = "wasm32")]
        {
            let m = "POST";
            let id = unsafe { agon_fetch(m.as_ptr(), m.len(), url.as_ptr(), url.len(), body.as_ptr(), body.len(), token.as_ptr(), token.len()) };
            Fetch { id }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (url, body, token);
            Fetch { done: Some((-1, OFFLINE.to_string())) }
        }
    }

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

impl Drop for Fetch {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_drop(self.id)
        };
    }
}

pub enum StreamItem {
    Msg(String),
    Empty,
    Closed,
}

pub struct Stream {
    #[cfg(target_arch = "wasm32")]
    id: i32,
}

impl Stream {
    pub fn open(url: &str, body: &str, token: &str) -> Stream {
        #[cfg(target_arch = "wasm32")]
        {
            let id = unsafe { agon_watch(url.as_ptr(), url.len(), body.as_ptr(), body.len(), token.as_ptr(), token.len()) };
            Stream { id }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (url, body, token);
            Stream {}
        }
    }

    pub fn next(&mut self) -> StreamItem {
        #[cfg(target_arch = "wasm32")]
        {
            let n = unsafe { agon_watch_next(self.id) };
            if n == -1 {
                return StreamItem::Empty;
            }
            if n < 0 {
                return StreamItem::Closed;
            }
            let mut buf = vec![0u8; n as usize];
            unsafe { agon_watch_copy(self.id, buf.as_mut_ptr()) };
            StreamItem::Msg(String::from_utf8_lossy(&buf).into_owned())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            StreamItem::Closed
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_watch_close(self.id)
        };
    }
}

pub struct Transport {
    #[cfg(target_arch = "wasm32")]
    id: i32,
}

impl Transport {
    pub fn open(url: &str, hash: &str) -> Transport {
        #[cfg(target_arch = "wasm32")]
        {
            let id = unsafe { agon_wt_open(url.as_ptr(), url.len(), hash.as_ptr(), hash.len()) };
            Transport { id }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (url, hash);
            Transport {}
        }
    }

    pub fn state(&self) -> i32 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { agon_wt_state(self.id) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            2
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

impl Drop for Transport {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            agon_wt_close(self.id)
        };
    }
}
