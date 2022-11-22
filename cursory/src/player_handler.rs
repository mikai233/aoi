use futures::SinkExt;
use protobuf::{MessageDyn, MessageField};
use protobuf::reflect::MessageDescriptor;

use protocol::mapper::cast;
use protocol::test::{LoginReq, LoginResp, PlayerMoveNotify};

use crate::message::WorldMessage::{PlayerLogin, PlayerMove};
use crate::message::WorldMessageWrap;
use crate::player::{Player, random_color};

pub fn proto_name(descriptor: MessageDescriptor) -> String {
    descriptor.name().to_string()
}

pub async fn handle_login_req(player: &mut Player, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let msg = cast::<LoginReq>(msg)?;
    player.player_id = msg.player_id;
    let mut rsp = LoginResp::new();
    rsp.player_id = player.player_id;
    let color = random_color();
    player.state.color = color;
    rsp.color = MessageField::some(player.state.color.clone());

    player.proto_sender.send(Box::new(rsp)).unwrap();
    player.world_sender.send(WorldMessageWrap::new(player.player_id, PlayerLogin(player.self_sender.clone(), player.proto_sender.clone(), player.state.clone())))?;
    Ok(())
}

pub async fn handle_move_notify(player: &mut Player, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let move_notify = cast::<PlayerMoveNotify>(msg)?;
    player.state.player_state = move_notify.state.clone().unwrap();

    player.world_sender.send(WorldMessageWrap::new(player.player_id, PlayerMove(move_notify)))?;
    Ok(())
}
