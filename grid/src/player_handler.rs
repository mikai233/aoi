use log::info;
use protobuf::{MessageDyn, MessageField};

use protocol::mapper::cast;
use protocol::test::{LoginReq, LoginResp};

use crate::event::ReceiveTimeoutEvent;
use crate::message::{EventMessage, KickOutReason, PlayerLoginData, WorldMessage, WorldMessageWrap};
use crate::player::{Player, PlayerSender, random_color};

pub async fn handle_world_kick_out(player: &mut Player, world_id: i32, reason: KickOutReason) -> anyhow::Result<()> {
    player.stop();
    Ok(())
}

pub async fn handle_event(player: &mut Player, event: EventMessage) -> anyhow::Result<()> {
    let event = event.0;
    if event.to_string() == ReceiveTimeoutEvent.to_string() {
        player.stop();
    }
    Ok(())
}

pub async fn handle_login_req(player: &mut Player, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let req = cast::<LoginReq>(msg)?;
    info!("player:{} handle login req",req.player_id);
    player.player_id = req.player_id;
    player.state.color = random_color();
    let wrap = WorldMessageWrap::new(player.player_id, WorldMessage::PlayerLogin(PlayerLoginData {
        sender: PlayerSender {
            player: player.player_sender.clone(),
            proto: player.proto_sender.clone(),
        },
        state: player.state.clone(),
    }));
    let _ = player.world_sender.send(wrap);
    let mut rsp = LoginResp::new();
    rsp.player_id = player.player_id;
    rsp.color = MessageField::some(player.state.color.clone());
    let _ = player.proto_sender.send(Box::new(rsp));
    Ok(())
}

pub async fn handle_move_req(player: &mut Player, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let wrap=WorldMessageWrap::new(player.player_id,WorldMessage::PlayerMove(msg));
    let _ = player.world_sender.send(wrap);
    Ok(())
}