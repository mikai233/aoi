use std::fmt::{Debug, Display};

use protobuf::MessageDyn;

use crate::player::State;

pub type ProtoMessage = Box<dyn MessageDyn>;

pub type PlayerMessageSender = tokio::sync::mpsc::UnboundedSender<PlayerMessageWrap>;
pub type PlayerMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<PlayerMessageWrap>;

pub type WorldMessageSender = tokio::sync::mpsc::UnboundedSender<WorldMessageWrap>;
pub type WorldMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<WorldMessageWrap>;

pub type ProtoMessageSender = tokio::sync::mpsc::UnboundedSender<ProtoMessage>;
pub type ProtoMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<ProtoMessage>;

pub struct WorldProtoMessage(pub i32, pub Box<dyn MessageDyn>);

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
    PlayerLogin(PlayerMessageSender, ProtoMessageSender, State),
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