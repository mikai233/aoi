use std::collections::hash_map::Entry;

use actix::Handler;
use log::{error, info, warn};

use crate::message::{PlayerLogin, SessionExpired, WorldProtoMessage};
use crate::world::WorldActor;
use crate::world_proto_handler::WORLD_PROTO_HANDLERS;

impl Handler<WorldProtoMessage> for WorldActor {
    type Result = ();

    fn handle(&mut self, msg: WorldProtoMessage, ctx: &mut Self::Context) -> Self::Result {
        let player_id = msg.0;
        let msg = msg.1;
        let msg_name = msg.descriptor_dyn().name().to_string();
        info!("world:{} receive player:{} msg:{}", self.world_id,player_id, msg_name);
        match WORLD_PROTO_HANDLERS.get(&msg_name) {
            None => {
                warn!("world:{} msg:{} handle not found", self.world_id, msg_name);
            }
            Some(handler) => {
                match handler(self, ctx, player_id, msg) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "world:{} handle msg:{} err:{}",
                            self.world_id, msg_name, err
                        );
                    }
                };
            }
        };
    }
}

impl Handler<PlayerLogin> for WorldActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerLogin, ctx: &mut Self::Context) -> Self::Result {
        let player_id = msg.0;
        let addr = msg.1;
        let entry = self.sessions.entry(player_id);
        match entry {
            Entry::Occupied(mut o) => {
                o.get().do_send(SessionExpired);
                o.insert(addr);
            }
            Entry::Vacant(v) => {
                v.insert(addr);
            }
        }
        info!("player:{} login to world:{}", player_id, self.world_id);
    }
}
