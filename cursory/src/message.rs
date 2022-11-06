use std::fmt::{Debug, Display, Formatter};

use actix::{Actor, Message};
use actix::dev::{MessageResponse, OneshotSender};
use protobuf::MessageDyn;

use crate::player::{PlayerMessageSender, PlayerState, ProtoMessageSender};

pub struct PoisonPill;

impl Message for PoisonPill {
    type Result = ();
}

pub struct PlayerProtoMessage(pub Box<dyn MessageDyn>);

impl Message for PlayerProtoMessage {
    type Result = ();
}

pub struct WorldProtoMessage(pub i32, pub Box<dyn MessageDyn>);

impl Message for WorldProtoMessage {
    type Result = ();
}

impl Display for PoisonPill {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct SessionExpired;

impl Message for SessionExpired {
    type Result = ();
}

pub struct Response(pub Box<dyn MessageDyn>);

impl Message for Response {
    type Result = ();
}

#[derive(Message)]
#[rtype(result = "PlayerStateAns")]
pub struct PlayerStateAsk;

#[derive(Debug)]
pub struct PlayerStateAns(pub PlayerState);

impl<A, M> MessageResponse<A, M> for PlayerStateAns
    where A: Actor,
          M: Message<Result=PlayerStateAns> {
    fn handle(self, ctx: &mut A::Context, tx: Option<OneshotSender<PlayerStateAns>>) {
        if let Some(tx) = tx {
            tx.send(self).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct WorldMessageWrap {
    pub player_id: i32,
    pub message: WorldMessage,
}

impl WorldMessageWrap {
    pub fn new(player_id: i32, message: WorldMessage) -> Self {
        Self {
            player_id,
            message,
        }
    }
}

#[derive(Debug)]
pub enum WorldMessage {
    PlayerLogin(PlayerMessageSender, ProtoMessageSender, PlayerState),
    PlayerLogout,
    PlayerMove(Box<dyn MessageDyn>),
    Proto(Box<dyn MessageDyn>),
}

#[derive(Debug)]
pub struct PlayerMessageWrap {
    pub world_id: i32,
    pub message: PlayerMessage,
}

impl PlayerMessageWrap {
    pub fn new(world_id: i32, message: PlayerMessage) -> Self {
        Self {
            world_id,
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerMessage {}