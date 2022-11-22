use log::{info, warn};
use protobuf::{MessageDyn, MessageField};

use protocol::mapper::cast;
use protocol::test::{PlayerMoveNotify, SCOtherPlayersStateNotify, SCPlayerEnterNotify};
use protocol::test::scother_players_state_notify::Bundle;
use protocol::test::SCPlayerMoveNotify;

use crate::player::{PlayerMessageSender, ProtoMessageSender, State};
use crate::world::World;

async fn handle_move_notify(world: &mut World, player_id: i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let msg = cast::<PlayerMoveNotify>(msg)?;
    if let Some(state) = world.player_state.get_mut(&player_id) {
        state.player_state = msg.state.clone().unwrap();
    } else {
        warn!("player:{} not found in world:{}",player_id,world.world_id);
    }
    for player in world.sessions.values() {
        let mut notify = SCPlayerMoveNotify::new();
        notify.state = msg.state.clone();
        notify.player_id = msg.player_id;
        let _ = player.1.send(Box::new(notify));
    }
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
