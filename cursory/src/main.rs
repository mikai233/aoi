use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::*;
use log::{error, info};
use tokio::net::TcpListener;

use crate::server::new_client;
use crate::world::WorldActor;

mod server;
mod player;
mod message;
mod world;
mod player_handler;
mod world_handler;

#[derive(Message)]
#[rtype(result = "Responses")]
enum Messages {
    Ping,
    Pong,
}

enum Responses {
    GotPing,
    GotPong,
}

impl<A, M> MessageResponse<A, M> for Responses
    where
        A: Actor,
        M: Message<Result=Responses>,
{
    fn handle(self, ctx: &mut A::Context, tx: Option<OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = ":8899";
    let listener = TcpListener::bind(addr).await?;
    info!("server listening on: {}",addr);
    
    loop {
        tokio::select! {
            c = listener.accept() => {
                match c {
                    Ok((stream, socket_addr)) => {
                        match new_client(stream).await {
                            Ok(_) => {
                                info!("client:{} disconnected",socket_addr);
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
