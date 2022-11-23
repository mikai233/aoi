use crate::event::ReceiveTimeoutEvent;
use crate::message::{EventMessage, KickOutReason};
use crate::player::Player;

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