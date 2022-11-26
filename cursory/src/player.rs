use protobuf::MessageDyn;
use rand::{Rng, thread_rng};

use protocol::test::{Color, PlayerState};

use crate::message::PlayerMessageWrap;
use crate::world::WorldMessageSender;

pub type PlayerMessageSender = tokio::sync::mpsc::UnboundedSender<PlayerMessageWrap>;
pub type ProtoMessageSender = tokio::sync::mpsc::UnboundedSender<Box<dyn MessageDyn>>;

pub struct Player {
    pub player_id: i32,
    pub self_sender: PlayerMessageSender,
    pub proto_sender: ProtoMessageSender,
    pub state: State,
    pub world_sender: WorldMessageSender,
}

impl Player {
    pub fn new(self_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, world_sender: WorldMessageSender) -> Self {
        Self {
            player_id: 0,
            proto_sender,
            self_sender,
            world_sender,
            state: State::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct State {
    pub player_state: PlayerState,
    pub color: Color,
}

pub fn random_color() -> Color {
    let mut thread_rng = thread_rng();
    let r = thread_rng.gen_range(0..1000) as f32 / 1000.;
    let g = thread_rng.gen_range(0..1000) as f32 / 1000.;
    let b = thread_rng.gen_range(0..1000) as f32 / 1000.;
    let mut color = Color::new();
    color.r = r;
    color.g = g;
    color.b = b;
    color
}
