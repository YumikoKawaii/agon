use crate::net::{
    native::Transport,
    packet::{Packet, PACKET_LEN},
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LinkState {
    Connecting,
    Open,
    Closed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Control {
    Start,
    End { winner: usize },
}

pub struct Link {
    transport: Transport,
}

impl Link {
    pub fn open(url: &str, ticket: &str, cert_hash: &str) -> Link {
        Link { transport: Transport::open(&format!("{url}?ticket={ticket}"), cert_hash) }
    }

    pub fn state(&self) -> LinkState {
        match self.transport.state() {
            0 => LinkState::Connecting,
            1 => LinkState::Open,
            _ => LinkState::Closed,
        }
    }

    pub fn send(&self, p: &Packet) {
        self.transport.send(&p.encode());
    }

    pub fn recv(&self) -> Vec<Packet> {
        let mut out = Vec::new();
        let mut buf = [0u8; PACKET_LEN];
        loop {
            let n = self.transport.recv(&mut buf);
            if n == 0 {
                return out;
            }
            out.extend(Packet::decode(&buf[..n]));
        }
    }

    pub fn control(&self) -> Vec<Control> {
        let mut out = Vec::new();
        while let Some(line) = self.transport.line() {
            if line == "start" {
                out.push(Control::Start);
            } else if let Some(w) = line.strip_prefix("end ").and_then(|w| w.parse().ok()) {
                out.push(Control::End { winner: w });
            }
        }
        out
    }
}
