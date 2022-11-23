use crate::message::PlayerLoginData;
use crate::world::World;

pub async fn handle_player_login(world: &mut World, player_id: i32, player_login_data: PlayerLoginData) -> anyhow::Result<()> {
    world.add_player(player_id, player_login_data);
    Ok(())
}