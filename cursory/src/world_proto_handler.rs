use std::collections::HashMap;

use lazy_static::lazy_static;
use protobuf::{MessageDyn, MessageFull};

use protocol::mapper::cast;
use protocol::test::{MoveStartReq, MoveStopReq, SCMoveStart, SCMoveStop};

use crate::message::PlayerProtoMessage;
use crate::world::WorldActor;

type WorldProtoHandler = fn(&mut WorldActor, i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()>;

lazy_static! {
    pub static ref WORLD_PROTO_HANDLERS: HashMap<String, WorldProtoHandler> = register_handlers();
}

fn register_handlers() -> HashMap<String, WorldProtoHandler> {
    let mut m = HashMap::new();
    m.insert(
        MoveStartReq::descriptor().name().to_string(),
        handle_move_start_req as WorldProtoHandler,
    );
    m.insert(
        MoveStopReq::descriptor().name().to_string(),
        handle_move_stop_req as WorldProtoHandler,
    );
    m
}

fn handle_move_start_req(
    world: &mut WorldActor,
    player_id: i32,
    msg: Box<dyn MessageDyn>,
) -> anyhow::Result<()> {
    let msg = cast::<MoveStartReq>(msg)?;
    for player in world.sessions.values() {
        let mut notify = SCMoveStart::new();
        notify.cmd = msg.cmd;
        notify.velocity = msg.velocity;
        notify.player_id = player_id;
        player.do_send(PlayerProtoMessage(Box::new(notify)));
    }
    Ok(())
}

fn handle_move_stop_req(
    world: &mut WorldActor,
    player_id: i32,
    msg: Box<dyn MessageDyn>,
) -> anyhow::Result<()> {
    let msg = cast::<MoveStopReq>(msg)?;
    for player in world.sessions.values() {
        let mut notify = SCMoveStop::new();
        notify.player_id = player_id;
        notify.location = msg.location.clone();
        player.do_send(PlayerProtoMessage(Box::new(notify)));
    }
    Ok(())
}
