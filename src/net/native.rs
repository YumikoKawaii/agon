use std::{
    env,
    io::Read,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, OnceLock,
    },
    thread,
};

use tokio::{runtime::Runtime, sync::mpsc as tmpsc, task::JoinHandle};
use ureq::Agent;
use wtransport::{tls::Sha256Digest, ClientConfig, Endpoint};

const DEFAULT_SERVER: &str = "http://localhost:7777";
const ENVELOPE_HEADER: usize = 5;
const END_STREAM: u8 = 2;
const DATAGRAM_BACKLOG: usize = 64;

struct Net {
    base: String,
    agent: Agent,
    runtime: Runtime,
}

fn net() -> &'static Net {
    static NET: OnceLock<Net> = OnceLock::new();
    NET.get_or_init(|| Net {
        base: env::var("AGON_SERVER").unwrap_or_else(|_| DEFAULT_SERVER.to_string()),
        agent: Agent::config_builder().http_status_as_error(false).build().into(),
        runtime: Runtime::new().expect("can't start network runtime"),
    })
}

fn request(path: &str, content_type: &str, token: &str) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
    let n = net();
    let mut req = n.agent.post(format!("{}{}", n.base, path)).header("Content-Type", content_type).header("Connect-Protocol-Version", "1");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    req
}

pub struct Fetch {
    rx: Receiver<(i32, String)>,
}

impl Fetch {
    pub fn post(path: &str, body: &str, token: &str) -> Fetch {
        let (tx, rx) = mpsc::channel();
        let req = request(path, "application/json", token);
        let body = if body.is_empty() { "{}".to_string() } else { body.to_string() };
        thread::spawn(move || {
            let out = match req.send(body) {
                Ok(mut res) => {
                    let status = res.status().as_u16() as i32;
                    (status, res.body_mut().read_to_string().unwrap_or_default())
                }
                Err(_) => (-1, String::new()),
            };
            let _ = tx.send(out);
        });
        Fetch { rx }
    }

    pub fn poll(&mut self) -> Option<(i32, String)> {
        match self.rx.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some((-1, String::new())),
        }
    }
}

pub enum StreamItem {
    Msg(String),
    Empty,
    Closed,
}

pub struct Stream {
    rx: Receiver<String>,
    stop: Arc<AtomicBool>,
}

fn read_envelopes(mut reader: impl Read, tx: mpsc::Sender<String>, stop: &AtomicBool) {
    let mut header = [0u8; ENVELOPE_HEADER];
    while !stop.load(Ordering::Relaxed) {
        if reader.read_exact(&mut header).is_err() {
            return;
        }
        let len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;
        let mut msg = vec![0u8; len];
        if reader.read_exact(&mut msg).is_err() || header[0] & END_STREAM != 0 {
            return;
        }
        if tx.send(String::from_utf8_lossy(&msg).into_owned()).is_err() {
            return;
        }
    }
}

impl Stream {
    pub fn open(path: &str, body: &str, token: &str) -> Stream {
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let req = request(path, "application/connect+json", token);
        let json = body.as_bytes();
        let mut envelope = Vec::with_capacity(ENVELOPE_HEADER + json.len());
        envelope.push(0);
        envelope.extend_from_slice(&(json.len() as u32).to_be_bytes());
        envelope.extend_from_slice(json);
        thread::spawn(move || {
            let Ok(res) = req.send(envelope) else { return };
            if res.status().is_success() {
                read_envelopes(res.into_body().into_reader(), tx, &flag);
            }
        });
        Stream { rx, stop }
    }

    pub fn next(&mut self) -> StreamItem {
        match self.rx.try_recv() {
            Ok(m) => StreamItem::Msg(m),
            Err(TryRecvError::Empty) => StreamItem::Empty,
            Err(TryRecvError::Disconnected) => StreamItem::Closed,
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

const CONNECTING: u8 = 0;
const OPEN: u8 = 1;
const CLOSED: u8 = 2;

pub struct Transport {
    state: Arc<AtomicU8>,
    out: tmpsc::UnboundedSender<Vec<u8>>,
    datagrams: Receiver<Vec<u8>>,
    lines: Receiver<String>,
    task: JoinHandle<()>,
}

fn digest(hex: &str) -> Option<Sha256Digest> {
    let bytes: Vec<u8> = (0..hex.len()).step_by(2).filter_map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok()).collect();
    Some(Sha256Digest::new(bytes.try_into().ok()?))
}

impl Transport {
    pub fn open(url: &str, hash: &str) -> Transport {
        let state = Arc::new(AtomicU8::new(CONNECTING));
        let (out, mut out_rx) = tmpsc::unbounded_channel::<Vec<u8>>();
        let (dgram_tx, datagrams) = mpsc::sync_channel(DATAGRAM_BACKLOG);
        let (line_tx, lines) = mpsc::channel();
        let url = url.to_string();
        let hash = digest(hash);
        let st = state.clone();
        let task = net().runtime.spawn(async move {
            let run = async {
                let config = ClientConfig::builder().with_bind_default().with_server_certificate_hashes(hash).build();
                let conn = Endpoint::client(config).ok()?.connect(url).await.ok()?;
                st.store(OPEN, Ordering::Relaxed);
                let mut ctrl = conn.accept_uni().await.ok()?;
                let mut pending = String::new();
                let mut buf = [0u8; 256];
                loop {
                    tokio::select! {
                        d = conn.receive_datagram() => {
                            let d = d.ok()?;
                            let _ = dgram_tx.try_send(d.payload().to_vec());
                        }
                        n = ctrl.read(&mut buf) => {
                            let n = n.ok()??;
                            pending.push_str(&String::from_utf8_lossy(&buf[..n]));
                            while let Some(i) = pending.find('\n') {
                                let line: String = pending.drain(..=i).collect();
                                let _ = line_tx.send(line.trim_end().to_string());
                            }
                        }
                        p = out_rx.recv() => {
                            let _ = conn.send_datagram(p?);
                        }
                    }
                }
            };
            let _: Option<()> = run.await;
            st.store(CLOSED, Ordering::Relaxed);
        });
        Transport { state, out, datagrams, lines, task }
    }

    pub fn state(&self) -> i32 {
        match self.state.load(Ordering::Relaxed) {
            CONNECTING => 0,
            OPEN => 1,
            _ => 2,
        }
    }

    pub fn send(&self, b: &[u8]) {
        let _ = self.out.send(b.to_vec());
    }

    pub fn recv(&self, buf: &mut [u8]) -> usize {
        match self.datagrams.try_recv() {
            Ok(d) if d.len() <= buf.len() => {
                buf[..d.len()].copy_from_slice(&d);
                d.len()
            }
            _ => 0,
        }
    }

    pub fn line(&self) -> Option<String> {
        self.lines.try_recv().ok()
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.task.abort();
    }
}
