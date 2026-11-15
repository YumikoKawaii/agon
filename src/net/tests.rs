use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::net::{
    api::{Api, Pending},
    link::{Control, Link, LinkState},
    Packet,
};

const WAIT: Duration = Duration::from_millis(20);
const TRIES: usize = 500;

fn wait<T: nanoserde::DeJson>(mut p: Pending<T>) -> T {
    for _ in 0..TRIES {
        if let Some(r) = p.poll() {
            return r.unwrap_or_else(|e| panic!("rpc failed: {}", e.message));
        }
        sleep(WAIT);
    }
    panic!("rpc timed out");
}

fn until<T>(mut f: impl FnMut() -> Option<T>) -> T {
    for _ in 0..TRIES {
        if let Some(v) = f() {
            return v;
        }
        sleep(WAIT);
    }
    panic!("condition timed out");
}

#[test]
#[ignore]
fn online_match() {
    let tag = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000;
    let anon = Api::default();
    let host = wait(anon.register(&format!("h{tag}"), "secret1"));
    let guest = wait(anon.register(&format!("g{tag}"), "secret1"));
    let (ha, ga) = (Api::with_token(host.token), Api::with_token(guest.token));
    let room = wait(ha.create_room()).room;
    let mut watch = ha.watch_room(&room.id);
    wait(ga.join_room(&room.id));
    until(|| watch.poll().0.filter(|r| r.guest.username == guest.user.username));
    wait(ha.set_ready(&room.id, true));
    let started = wait(ga.set_ready(&room.id, true)).room;
    assert!(started.playing());
    let he = wait(ha.endpoint(&started.match_id));
    let ge = wait(ga.endpoint(&started.match_id));
    let hl = Link::open(&he.url, &he.ticket, &he.cert_hash);
    until(|| (hl.state() == LinkState::Open).then_some(()));
    let gl = Link::open(&ge.url, &ge.ticket, &ge.cert_hash);
    until(|| hl.control().contains(&Control::Start).then_some(()));
    until(|| gl.control().contains(&Control::Start).then_some(()));
    let sent = Packet { tick: 7, hp: 42, ..Packet::default() };
    let got = until(|| {
        hl.send(&sent);
        gl.recv().into_iter().last()
    });
    assert_eq!((got.tick, got.hp), (7, 42));
    hl.send(&Packet { tick: 8, deaths: 3, ..Packet::default() });
    let end = until(|| gl.control().into_iter().find(|c| matches!(c, Control::End { .. })));
    assert_eq!(end, Control::End { winner: 1 });
}
