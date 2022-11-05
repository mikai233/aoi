use std::fmt::{Debug, Display, Formatter};

use actix::{Actor, Addr, Message};
use actix::dev::{MessageResponse, OneshotSender};
use protobuf::MessageDyn;

use crate::player::{Location, PlayerActor};

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

pub struct PlayerLogin(pub i32, pub Addr<PlayerActor>);

impl Message for PlayerLogin {
    type Result = ();
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
#[rtype(result = "LocationAns")]
pub struct LocationAsk;

#[derive(Debug)]
pub struct LocationAns(pub Location);

impl<A, M> MessageResponse<A, M> for LocationAns
    where A: Actor,
          M: Message<Result=LocationAns> {
    fn handle(self, ctx: &mut A::Context, tx: Option<OneshotSender<LocationAns>>) {
        if let Some(tx) = tx {
            tx.send(self).unwrap();
        }
    }
}