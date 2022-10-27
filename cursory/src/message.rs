use std::fmt::{Display, Formatter};

use actix::{Addr, Message};
use protobuf::MessageDyn;

use crate::player::PlayerActor;

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
