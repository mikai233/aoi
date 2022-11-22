use anyhow::anyhow;
use log::info;
use protobuf::{MessageDyn, MessageField};

use protocol::mapper::cast;
use protocol::test::{PlayerMoveNotify, SCOtherPlayersStateNotify, SCPlayerEnterNotify};
use protocol::test::scother_players_state_notify::Bundle;
use protocol::test::SCPlayerMoveNotify;

use crate::player::{PlayerMessageSender, ProtoMessageSender, State};
use crate::world::World;

pub async fn handle_move_notify(world: &mut World, player_id: i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let move_notify = cast::<PlayerMoveNotify>(msg)?;
    let state = world.player_state.get_mut(&player_id).ok_or(anyhow!("player:{} state not found",player_id))?;
    state.player_state = move_notify.state.clone().unwrap();
    let mut notify = SCPlayerMoveNotify::new();
    notify.state = move_notify.state.clone();
    notify.player_id = player_id;
    world.broad_cast_all(Box::new(notify));
    Ok(())
}

pub async fn handle_player_login(world: &mut World, player_id: i32, player_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, state: State) -> anyhow::Result<()> {
    world.sessions.insert(player_id, (player_sender, proto_sender));
    world.player_state.insert(player_id, state.clone());
    let mut notify = SCPlayerEnterNotify::new();
    notify.player_id = player_id;
    notify.color = MessageField::some(state.color);
    world.broad_cast_others(player_id, Box::new(notify));
    let mut others_state_notify = SCOtherPlayersStateNotify::new();
    for (&id, state) in &world.player_state {
        if id != player_id {
            let mut b = Bundle::new();
            b.player_id = id;
            b.state = MessageField::some(state.player_state.clone());
            b.color = MessageField::some(state.color.clone());
            others_state_notify.players.push(b);
        }
    }
    let sender = &world.sessions[&player_id];
    let _ = sender.1.send(Box::new(others_state_notify));
    info!("player {} login",player_id);
    Ok(())
}
