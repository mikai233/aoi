use std::collections::HashMap;

use actix::{AsyncContext, Context};
use lazy_static::lazy_static;
use protobuf::{MessageDyn, MessageFull};

use protocol::mapper::cast;
use protocol::test::{LoginReq, LoginResp, PlayerMoveNotify, SCPlayerEnterNotify};

use crate::message::{PlayerLogin, WorldProtoMessage};
use crate::player::PlayerActor;

type PlayerProtoHandler =
fn(&mut PlayerActor, &mut Context<PlayerActor>, msg: Box<dyn MessageDyn>) -> anyhow::Result<()>;

lazy_static! {
    pub static ref PLAYER_PROTO_HANDLERS: HashMap<String, PlayerProtoHandler> =
        { register_handlers() };
}

fn register_handlers() -> HashMap<String, PlayerProtoHandler> {
    let mut m = HashMap::new();
    m.insert(
        LoginReq::descriptor().name().to_string(),
        handle_login_req as PlayerProtoHandler,
    );
    m.insert(
        PlayerMoveNotify::descriptor().name().to_string(),
        handle_move_notify as PlayerProtoHandler,
    );
    m
}

fn handle_login_req(
    player: &mut PlayerActor,
    ctx: &mut Context<PlayerActor>,
    msg: Box<dyn MessageDyn>,
) -> anyhow::Result<()> {
    let msg = cast::<LoginReq>(msg)?;
    player.player_id = msg.player_id;
    let mut rsp = LoginResp::new();
    rsp.player_id = player.player_id;
    player.send(ctx, Box::new(rsp));
    player.world_pid.do_send(PlayerLogin(player.player_id, ctx.address()));
    let mut enter = SCPlayerEnterNotify::new();
    enter.player_id = player.player_id;
    player.world_pid.do_send(WorldProtoMessage(player.player_id, Box::new(enter)));
    Ok(())
}

fn handle_move_notify(
    player: &mut PlayerActor,
    ctx: &mut Context<PlayerActor>,
    msg: Box<dyn MessageDyn>,
) -> anyhow::Result<()> {
    let msg = cast::<PlayerMoveNotify>(msg)?;
    let world_msg = WorldProtoMessage(player.player_id, msg);

    player.world_pid.do_send(world_msg);
    Ok(())
}
