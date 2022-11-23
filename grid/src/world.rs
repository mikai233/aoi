use std::collections::HashMap;

use log::{error, info, warn};
use protobuf::MessageDyn;

use crate::message::{KickOutReason, PlayerLoginData, PlayerMessage, PlayerMessageWrap, WorldMessage, WorldMessageSender, WorldMessageWrap};
use crate::player::{PlayerSender, State};
use crate::world_handler::handle_player_login;

pub struct World {
    pub world_id: i32,
    pub sessions: HashMap<i32, PlayerSender>,
    pub player_states: HashMap<i32, State>,
}

impl World {
    pub fn new() -> Self {
        Self {
            world_id: 0,
            sessions: HashMap::new(),
            player_states: HashMap::new(),
        }
    }

    pub async fn handle_world_msg(&mut self, msg: WorldMessageWrap) -> anyhow::Result<()> {
        let player_id = msg.player_id;
        match msg.message {
            WorldMessage::PlayerLogin(data) => {
                handle_player_login(self, player_id, data).await?;
            }
            WorldMessage::PlayerLogout => {}
            WorldMessage::PlayerMove(_) => {}
            WorldMessage::Proto(_) => {}
        }
        Ok(())
    }

    pub fn broadcast_msg_to_all(&mut self, msg: Box<dyn MessageDyn>) {
        let mut remove_players = vec![];
        for (player_id, sender) in &self.sessions {
            if let Some(err) = sender.proto.send(msg.clone()).err() {
                warn!("broadcast message to player {} err {}, player session will be remove",player_id,err);
                remove_players.push(*player_id);
            }
        }
        self.remove_players(remove_players);
    }

    pub fn broadcast_msg_to_others(&mut self, current_player_id: i32, msg: Box<dyn MessageDyn>) {
        let mut remove_players = vec![];
        for (player_id, sender) in &self.sessions {
            if current_player_id == *player_id {
                continue;
            }
            if let Some(err) = sender.proto.send(msg.clone()).err() {
                warn!("broadcast message to player {} err {}, player session will be remove",player_id,err);
                remove_players.push(*player_id);
            }
        }
        self.remove_players(remove_players);
    }

    pub fn remove_players(&mut self, players: Vec<i32>) {
        for player_id in players {
            self.sessions.remove(&player_id);
            self.player_states.remove(&player_id);
            info!("player {} session removed from world {}",player_id,self.world_id);
        }
    }

    pub fn add_player(&mut self, player_id: i32, player_login_data: PlayerLoginData) {
        let session = self.sessions.get(&player_id);
        if let Some(sender) = session {
            let _ = sender.player.send(PlayerMessageWrap::new(self.world_id, PlayerMessage::KickOut(KickOutReason::MultiLogin("other player login with same account".to_string()))));
        }
        self.sessions.insert(player_id, player_login_data.sender);
        self.player_states.insert(player_id, player_login_data.state);
    }
}

pub fn start_world() -> WorldMessageSender {
    let world = World::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WorldMessageWrap>();
    tokio::spawn(async move {
        let mut world = world;
        loop {
            match rx.recv().await {
                None => {
                    //world dont stop
                }
                Some(message) => {
                    match world.handle_world_msg(message).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("world {} handle message error {}",world.world_id,err);
                        }
                    }
                }
            };
        }
    });
    tx
}