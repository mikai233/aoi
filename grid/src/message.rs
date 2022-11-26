use std::fmt::Debug;

use protobuf::MessageDyn;

use crate::player::{PlayerSender, State};
use crate::tick::ScheduleEvent;

pub type ProtoMessage = Box<dyn MessageDyn>;

pub type PlayerMessageSender = tokio::sync::mpsc::UnboundedSender<PlayerMessageWrap>;
pub type PlayerMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<PlayerMessageWrap>;

pub type EventMessageSender = tokio::sync::mpsc::UnboundedSender<EventMessage>;
pub type EventMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<EventMessage>;

pub type WorldMessageSender = tokio::sync::mpsc::UnboundedSender<WorldMessageWrap>;
pub type WorldMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<WorldMessageWrap>;

pub type ProtoMessageSender = tokio::sync::mpsc::UnboundedSender<ProtoMessage>;
pub type ProtoMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<ProtoMessage>;

pub struct WorldProtoMessage(pub i32, pub Box<dyn MessageDyn>);

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum WorldMessage {
    PlayerLogin(PlayerLoginData),
    PlayerLogout,
    PlayerMove(Box<dyn MessageDyn>),
    Proto(Box<dyn MessageDyn>),
}

#[derive(Debug, Clone)]
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
pub struct PlayerLoginData {
    pub sender: PlayerSender,
    pub state: State,
}

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    KickOut(KickOutReason),
    Event(Box<dyn ScheduleEvent>),
}

#[derive(Debug, Clone)]
pub enum KickOutReason {
    MultiLogin(String)
}

pub struct EventMessage(pub Box<dyn ScheduleEvent>);