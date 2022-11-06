use protobuf::MessageDyn;
use rand::{Rng, thread_rng};

use protocol::test::{Color, State};

use crate::message::PlayerMessageWrap;
use crate::world::WorldMessageSender;

pub type PlayerMessageSender = tokio::sync::mpsc::UnboundedSender<PlayerMessageWrap>;
pub type ProtoMessageSender = tokio::sync::mpsc::UnboundedSender<Box<dyn MessageDyn>>;

pub struct Player {
    pub player_id: i32,
    pub self_sender: PlayerMessageSender,
    pub proto_sender: ProtoMessageSender,
    pub state: PlayerState,
    pub world_sender: WorldMessageSender,
}

impl Player {
    pub fn new(self_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, world_sender: WorldMessageSender) -> Self {
        Self {
            player_id: 0,
            proto_sender,
            self_sender,
            world_sender,
            state: PlayerState::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PlayerState {
    pub x: f64,
    pub y: f64,
    pub state: State,
    pub color: Color,
}

pub fn random_color() -> Color {
    let mut thread_rng = thread_rng();
    let r = thread_rng.gen_range(0..1000) as f64 / 1000.;
    let g = thread_rng.gen_range(0..1000) as f64 / 1000.;
    let b = thread_rng.gen_range(0..1000) as f64 / 1000.;
    let mut color = Color::new();
    color.r = r;
    color.g = g;
    color.b = b;
    color
}
