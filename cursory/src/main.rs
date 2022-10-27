use actix::prelude::*;
use log::{error, info};
use tokio::net::TcpListener;

use crate::server::new_client;
use crate::world::WorldActor;

mod message;
mod player;
mod player_handler;
mod player_proto_handler;
mod server;
mod world;
mod world_handler;
mod world_proto_handler;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    let listener = TcpListener::bind(addr).await?;
    info!("server listening on: {}", addr);
    let world_actor = WorldActor::new();
    let world_pid = world_actor.start();
    loop {
        tokio::select! {
            c = listener.accept() => {
                match c {
                    Ok((stream, socket_addr)) => {
                        match new_client(stream, world_pid.clone()).await {
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
