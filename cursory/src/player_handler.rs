use actix::{ActorContext, Handler};
use log::{error, info, warn};

use crate::message::{LocationAns, LocationAsk, PlayerProtoMessage, PoisonPill, Response, SessionExpired};
use crate::player::PlayerActor;
use crate::player_proto_handler::PLAYER_PROTO_HANDLERS;

impl Handler<PlayerProtoMessage> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerProtoMessage, ctx: &mut Self::Context) -> Self::Result {
        info!("player:{} receive msg:{}", self.player_id, msg.0);
        let msg_name = msg.0.descriptor_dyn().name().to_string();
        match PLAYER_PROTO_HANDLERS.get(&msg_name) {
            None => {
                warn!(
                    "player:{} msg:{} handle not found",
                    self.player_id, msg_name
                );
            }
            Some(handler) => {
                match handler(self, ctx, msg.0) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "player:{} handle msg:{} err:{}",
                            self.player_id, msg_name, err
                        );
                    }
                };
            }
        };
    }
}

impl Handler<PoisonPill> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: PoisonPill, ctx: &mut Self::Context) -> Self::Result {
        info!("player:{} receive msg:{} stop self", self.player_id, msg);
        ctx.stop();
    }
}

impl Handler<SessionExpired> for PlayerActor {
    type Result = ();

    fn handle(&mut self, _: SessionExpired, ctx: &mut Self::Context) -> Self::Result {
        info!("player:{} session expired, stop self", self.player_id);
        ctx.stop();
    }
}

impl Handler<Response> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: Response, ctx: &mut Self::Context) -> Self::Result {
        self.send(ctx, msg.0);
    }
}

impl Handler<LocationAsk> for PlayerActor {
    type Result = LocationAns;

    fn handle(&mut self, msg: LocationAsk, ctx: &mut Self::Context) -> Self::Result {
        LocationAns(self.location.clone())
    }
}