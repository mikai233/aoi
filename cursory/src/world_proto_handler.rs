use std::collections::HashMap;

use actix::{Context, ContextFutureSpawner, WrapFuture};
use lazy_static::lazy_static;
use log::error;
use protobuf::{EnumOrUnknown, MessageDyn, MessageField, MessageFull};

use protocol::mapper::cast;
use protocol::test::{PlayerMoveNotify, SCPlayerEnterNotify, SCSyncPlayerLocationNotify, Vector2};
use protocol::test::SCPlayerMoveNotify;

use crate::message::{LocationAsk, Response};
use crate::world::WorldActor;

type WorldProtoHandler = fn(&mut WorldActor, &mut Context<WorldActor>, i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()>;

lazy_static! {
    pub static ref WORLD_PROTO_HANDLERS: HashMap<String, WorldProtoHandler> = register_handlers();
}

fn register_handlers() -> HashMap<String, WorldProtoHandler> {
    let mut m = HashMap::new();
    m.insert(PlayerMoveNotify::descriptor().name().to_string(), handle_move_notify as WorldProtoHandler);
    m.insert(SCPlayerEnterNotify::descriptor().name().to_string(), handle_player_enter as WorldProtoHandler);
    m
}

fn handle_move_notify(
    world: &mut WorldActor,
    ctx: &mut Context<WorldActor>,
    player_id: i32,
    msg: Box<dyn MessageDyn>,
) -> anyhow::Result<()> {
    let msg = cast::<PlayerMoveNotify>(msg)?;
    for player in world.sessions.values() {
        let mut notify = SCPlayerMoveNotify::new();
        notify.state = msg.state;
        notify.location = msg.location.clone();
        notify.player_id = msg.player_id;
        player.do_send(Response(Box::new(notify)));
    }
    Ok(())
}

fn handle_player_enter(world: &mut WorldActor, ctx: &mut Context<WorldActor>, player_id: i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let msg = cast::<SCPlayerEnterNotify>(msg)?;
    let current_player = &world.sessions[&player_id];
    for (id, addr) in &world.sessions {
        if player_id != *id {
            //notify other player you enter
            addr.do_send(Response(msg.clone()));
            //sync other player's location to you
            let current = current_player.clone();
            let other = addr.clone();
            let player_id = id.clone();
            Box::pin(async move {
                match other.send(LocationAsk).await {
                    Ok(ans) => {
                        let mut notify = SCSyncPlayerLocationNotify::new();
                        notify.player_id = player_id;
                        let mut v2 = Vector2::new();
                        v2.x = ans.0.x;
                        v2.y = ans.0.y;
                        notify.location = MessageField::some(v2);
                        notify.state = EnumOrUnknown::new(ans.0.state);
                        current.do_send(Response(Box::new(notify)));
                    }
                    Err(err) => {
                        error!("ask player:{} location err:{}",player_id,err);
                    }
                };
            }).into_actor(world).wait(ctx);
        }
    }
    Ok(())
}
