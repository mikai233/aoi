use std::collections::HashMap;

use log::{error, info};
use protobuf::MessageDyn;

use crate::message::{WorldMessage, WorldMessageWrap};
use crate::player::{PlayerMessageSender, ProtoMessageSender, State};
use crate::world_handler::handle_move_notify;
use crate::world_handler::handle_player_login;

pub type WorldMessageSender = tokio::sync::mpsc::UnboundedSender<WorldMessageWrap>;
pub type WorldMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<WorldMessageWrap>;

pub struct World {
    pub world_id: i32,
    pub sessions: HashMap<i32, (PlayerMessageSender, ProtoMessageSender)>,
    pub player_state: HashMap<i32, State>,
}

impl World {
    pub fn new() -> Self {
        Self {
            world_id: 0,
            sessions: HashMap::new(),
            player_state: HashMap::new(),
        }
    }
    pub fn broad_cast_all(&mut self, msg: Box<dyn MessageDyn>) {
        for x in self.sessions.values() {
            let _ = x.1.send(msg.clone());
        }
    }

    pub fn broad_cast_others(&mut self, player_id: i32, msg: Box<dyn MessageDyn>) {
        for (&id, tx) in &self.sessions {
            if id != player_id {
                let _ = tx.1.send(msg.clone());
            }
        }
    }
}

pub fn start_world(mut world_receiver: WorldMessageReceiver) {
    tokio::spawn(async move {
        let mut world = World::new();
        loop {
            match world_receiver.recv().await {
                None => {}
                Some(wrap) => {
                    match inner_handler(&mut world, wrap).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("world:{} handle msg err:{}",world.world_id,err);
                        }
                    }
                }
            };
        }
    });
}

async fn inner_handler(world: &mut World, wrap: WorldMessageWrap) -> anyhow::Result<()> {
    let player_id = wrap.player_id;
    match wrap.message {
        WorldMessage::PlayerLogin(player_sender, proto_sender, state) => {
            handle_player_login(world, player_id, player_sender, proto_sender, state).await?;
        }
        WorldMessage::PlayerLogout => {
            todo!()
        }
        WorldMessage::Proto(proto_msg) => {
            let desc = proto_msg.descriptor_dyn();
            let msg_name = desc.name();
            todo!()
        }
        WorldMessage::PlayerMove(move_notify) => {
            info!("{}",move_notify);
            handle_move_notify(world, player_id, move_notify).await?;
        }
    }
    Ok(())
}