use std::net::SocketAddr;

use futures::StreamExt;
use log::{error, info};
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::mapper::kcp_config;

use crate::message::{PlayerMessageWrap, ProtoMessage, WorldMessageSender};
use crate::player::Player;
use crate::world::start_world;

pub async fn start_server(addr: &str) -> anyhow::Result<()> {
    let cfg = kcp_config();
    let world_sender = start_world();
    let mut listener = tokio_kcp::KcpListener::bind(cfg, addr).await?;
    info!("server start at {}",addr);
    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream,addr)) => accept_connection(stream,addr,world_sender.clone()),
                    Err(err) => {
                        error!("server accept connection error {}",err);
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

fn accept_connection(stream: KcpStream, addr: SocketAddr, world_sender: WorldMessageSender) {
    let (player_tx, player_rx) = tokio::sync::mpsc::unbounded_channel::<PlayerMessageWrap>();
    let (proto_tx, proto_rx) = tokio::sync::mpsc::unbounded_channel::<ProtoMessage>();
    let player = Player::new(addr, player_tx, proto_tx, world_sender);
    let framed = Framed::new(stream, ProtoCodec::new(true));
    let (write, read) = framed.split();
    Player::start_receive_msg(player, read, player_rx);
    Player::start_write_msg(proto_rx, write);
    info!("accept new connection {}",addr);
}