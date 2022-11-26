use protobuf::MessageDyn;

use protocol::mapper::cast;
use protocol::test::PlayerMoveNotify;

use crate::message::PlayerLoginData;
use crate::world::World;

pub async fn handle_player_login(world: &mut World, player_id: i32, player_login_data: PlayerLoginData) -> anyhow::Result<()> {
    world.add_player(player_id, player_login_data);
    //todo sync other player's state
    Ok(())
}

pub async fn handle_player_move(world: &mut World, player_id: i32, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
    let notify = cast::<PlayerMoveNotify>(msg)?;
    world.move_player(player_id, notify.state.unwrap());
    Ok(())
}