use log::{error, info};

use protocol::mapper::kcp_config;

use crate::message::WorldMessageWrap;
use crate::server::new_client;
use crate::world::start_world;

mod message;
mod player;
mod server;
mod world;
mod player_handler;
mod world_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    let cfg = kcp_config();
    let mut listener = tokio_kcp::KcpListener::bind(cfg, addr).await?;
    info!("server listening on: {}", addr);
    let (world_sender, mut world_receiver) = tokio::sync::mpsc::unbounded_channel::<WorldMessageWrap>();
    start_world(world_receiver);
    loop {
        tokio::select! {
            c = listener.accept() => {
                match c {
                    Ok((stream, socket_addr)) => {
                        match new_client(stream, world_sender.clone()).await {
                            Ok(_) => {
                                info!("client:{} connected",socket_addr);
                            }
                            Err(err) => {
                                error!("{} disconnected with err: {}",socket_addr,err);
                            }
                        };
                    }
                    Err(err) => {
                        error!("server accept connection err:{}",err);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("signal ctrl c, close server");
                break;
            }
        }
    }
    Ok(())
}
