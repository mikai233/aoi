use crate::message::{PlayerMessageSender, ProtoMessageSender};
use crate::player::State;
use crate::world::World;

pub async fn handle_player_login(world: &mut World, player_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, state: State) -> anyhow::Result<()> {
    Ok(())
}