use log::warn;
use protobuf::{EnumOrUnknown, MessageDyn, MessageField};

use protocol::mapper::cast;
use protocol::test::{PlayerMoveNotify, SCOtherPlayersStateNotify, SCPlayerEnterNotify, Vector2};
use protocol::test::scother_players_state_notify::Bundle;
use protocol::test::SCPlayerMoveNotify;

use crate::player::{PlayerMessageSender, PlayerState, ProtoMessageSender};
use crate::world::World;

async fn handle_move_notify(world: &mut World, player_id: i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let msg = cast::<PlayerMoveNotify>(msg)?;
    if let Some(state) = world.player_state.get_mut(&player_id) {
        state.state = msg.state.unwrap();
        state.x = msg.location.x;
        state.y = msg.location.y;
    } else {
        warn!("player:{} not found in world:{}",player_id,world.world_id);
    }
    for player in world.sessions.values() {
        let mut notify = SCPlayerMoveNotify::new();
        notify.state = msg.state;
        notify.location = msg.location.clone();
        notify.player_id = msg.player_id;
        let _ = player.1.send(Box::new(notify));
    }
    Ok(())
}

pub async fn handle_player_login(world: &mut World, player_id: i32, player_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, state: PlayerState) -> anyhow::Result<()> {
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
            b.state = EnumOrUnknown::new(state.state.clone());
            b.color = MessageField::some(state.color.clone());
            let mut v = Vector2::new();
            v.x = state.x;
            v.y = state.y;
            b.location = MessageField::some(v);
            others_state_notify.players.push(b);
        }
    }
    let sender = &world.sessions[&player_id];
    let _ = sender.1.send(Box::new(others_state_notify));
    Ok(())
}
