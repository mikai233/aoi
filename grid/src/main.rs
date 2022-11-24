use crate::server::start_server;

mod player;
mod message;
mod world;
mod server;
mod world_handler;
mod player_handler;
mod tick;
mod event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    start_server(addr).await?;
    Ok(())
}
