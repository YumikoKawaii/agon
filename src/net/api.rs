use std::marker::PhantomData;

use nanoserde::{DeJson, SerJson};

use crate::net::ffi::{Fetch, Stream, StreamItem};

#[derive(Debug, Clone)]
pub struct ApiError {
    pub unauthenticated: bool,
    pub message: String,
}

#[derive(DeJson)]
struct ErrorBody {
    #[nserde(default)]
    code: String,
    #[nserde(default)]
    message: String,
}

impl ApiError {
    fn parse(status: i32, body: &str) -> ApiError {
        let parsed = ErrorBody::deserialize_json(body).ok();
        let message = match &parsed {
            Some(e) if !e.message.is_empty() => e.message.clone(),
            _ if status < 0 => "server unreachable".to_string(),
            _ => format!("error {status}"),
        };
        let unauthenticated = status == 401 || parsed.is_some_and(|e| e.code == "unauthenticated");
        ApiError { unauthenticated, message }
    }
}

pub struct Pending<T> {
    fetch: Fetch,
    out: PhantomData<T>,
}

impl<T: DeJson> Pending<T> {
    pub fn poll(&mut self) -> Option<Result<T, ApiError>> {
        let (status, body) = self.fetch.poll()?;
        if !(200..300).contains(&status) {
            return Some(Err(ApiError::parse(status, &body)));
        }
        Some(T::deserialize_json(&body).map_err(|e| ApiError { unauthenticated: false, message: format!("bad response: {e}") }))
    }
}

#[derive(SerJson)]
struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(SerJson)]
struct KitReq {
    kit: usize,
}

#[derive(SerJson)]
struct RoomReq<'a> {
    #[nserde(rename = "roomId")]
    room_id: &'a str,
}

#[derive(SerJson)]
struct ReadyReq<'a> {
    #[nserde(rename = "roomId")]
    room_id: &'a str,
    ready: bool,
}

#[derive(SerJson)]
struct MatchReq<'a> {
    #[nserde(rename = "matchId")]
    match_id: &'a str,
}

#[derive(DeJson, Default, Clone, Debug)]
pub struct User {
    #[nserde(default)]
    pub username: String,
    #[nserde(default)]
    pub kit: usize,
}

#[derive(DeJson, Debug)]
pub struct Auth {
    #[nserde(default)]
    pub token: String,
    #[nserde(default)]
    pub user: User,
}

#[derive(DeJson, Debug)]
pub struct UserReply {
    #[nserde(default)]
    pub user: User,
}

#[derive(DeJson, Default, Clone, Debug)]
pub struct Seat {
    #[nserde(default)]
    pub username: String,
    #[nserde(default)]
    pub kit: usize,
    #[nserde(default)]
    pub ready: bool,
}

#[derive(DeJson, Default, Clone, Debug)]
pub struct Room {
    #[nserde(default)]
    pub id: String,
    #[nserde(default)]
    pub host: Seat,
    #[nserde(default)]
    pub guest: Seat,
    #[nserde(default)]
    pub state: String,
    #[nserde(default, rename = "matchId")]
    pub match_id: String,
}

impl Room {
    pub fn waiting(&self) -> bool {
        self.state == "ROOM_STATE_WAITING"
    }

    pub fn playing(&self) -> bool {
        self.state == "ROOM_STATE_PLAYING"
    }

    pub fn closed(&self) -> bool {
        self.state == "ROOM_STATE_CLOSED"
    }

    pub fn seat_of(&self, username: &str) -> usize {
        if self.host.username == username {
            0
        } else {
            1
        }
    }

    pub fn seat(&self, i: usize) -> &Seat {
        if i == 0 {
            &self.host
        } else {
            &self.guest
        }
    }
}

#[derive(DeJson, Debug)]
pub struct RoomReply {
    #[nserde(default)]
    pub room: Room,
}

#[derive(DeJson, Debug)]
pub struct Empty {}

#[derive(DeJson, Debug)]
pub struct Endpoint {
    #[nserde(default)]
    pub url: String,
    #[nserde(default, rename = "certHash")]
    pub cert_hash: String,
    #[nserde(default)]
    pub ticket: String,
    #[nserde(default)]
    pub seat: usize,
}

#[derive(Clone, Default)]
pub struct Api {
    token: String,
}

fn url(rpc: &str) -> String {
    format!("/agon.v1.{rpc}")
}

impl Api {
    pub fn with_token(token: String) -> Api {
        Api { token }
    }

    fn call<T: DeJson>(&self, rpc: &str, body: String) -> Pending<T> {
        Pending { fetch: Fetch::post(&url(rpc), &body, &self.token), out: PhantomData }
    }

    pub fn register(&self, username: &str, password: &str) -> Pending<Auth> {
        self.call("AuthService/Register", Credentials { username, password }.serialize_json())
    }

    pub fn login(&self, username: &str, password: &str) -> Pending<Auth> {
        self.call("AuthService/Login", Credentials { username, password }.serialize_json())
    }

    pub fn logout(&self) -> Pending<Empty> {
        self.call("AuthService/Logout", "{}".into())
    }

    pub fn set_kit(&self, kit: usize) -> Pending<UserReply> {
        self.call("UserService/SetKit", KitReq { kit }.serialize_json())
    }

    pub fn create_room(&self) -> Pending<RoomReply> {
        self.call("RoomService/CreateRoom", "{}".into())
    }

    pub fn join_room(&self, room_id: &str) -> Pending<RoomReply> {
        self.call("RoomService/JoinRoom", RoomReq { room_id }.serialize_json())
    }

    pub fn get_room(&self, room_id: &str) -> Pending<RoomReply> {
        self.call("RoomService/GetRoom", RoomReq { room_id }.serialize_json())
    }

    pub fn leave_room(&self, room_id: &str) -> Pending<Empty> {
        self.call("RoomService/LeaveRoom", RoomReq { room_id }.serialize_json())
    }

    pub fn set_ready(&self, room_id: &str, ready: bool) -> Pending<RoomReply> {
        self.call("RoomService/SetReady", ReadyReq { room_id, ready }.serialize_json())
    }

    pub fn endpoint(&self, match_id: &str) -> Pending<Endpoint> {
        self.call("MatchService/GetEndpoint", MatchReq { match_id }.serialize_json())
    }

    pub fn watch_room(&self, room_id: &str) -> RoomWatch {
        RoomWatch { stream: Stream::open(&url("RoomService/WatchRoom"), &RoomReq { room_id }.serialize_json(), &self.token) }
    }
}

pub struct RoomWatch {
    stream: Stream,
}

impl RoomWatch {
    pub fn poll(&mut self) -> (Option<Room>, bool) {
        let mut latest = None;
        loop {
            match self.stream.next() {
                StreamItem::Msg(m) => {
                    if let Ok(r) = RoomReply::deserialize_json(&m) {
                        latest = Some(r.room);
                    }
                }
                StreamItem::Empty => return (latest, false),
                StreamItem::Closed => return (latest, true),
            }
        }
    }
}
