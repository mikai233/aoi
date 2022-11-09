use std::time::Duration;

use futures::{SinkExt, StreamExt};
use log::{error, info};
use protobuf::{Enum, MessageDyn};
use rand::{random, Rng};
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::mapper::kcp_config;
use protocol::test::LoginReq;

use crate::client::{Client, ClientMessage};

mod client;

pub const PLAYER_COUNT: usize = 10;

pub const HORIZONTAL_BOUNDARY: f64 = 1000.;

pub const VERTICAL_BOUNDARY: f64 = 1000.;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    let mut clients = vec![];
    for _ in 0..PLAYER_COUNT {
        let c = tokio::spawn(start_client(addr));
        clients.push(c);
    }
    for c in clients {
        match c.await {
            Ok(_) => {}
            Err(error) => {
                error!("{}",error);
            }
        }
    }
    Ok(())
}

async fn start_client(addr: &str) {
    let cfg = kcp_config();
    let stream = tokio_kcp::KcpStream::connect(&cfg, addr.parse().unwrap()).await.unwrap();

    let framed = Framed::new(stream, ProtoCodec::new(false));
    let (sink, mut stream) = framed.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut client = Client::new(sink, tx.clone(), rx);
    let player_id = random();
    info!("client:{} started", player_id);
    client.player_id = player_id;
    let mut login = LoginReq::new();
    login.player_id = player_id;
    client.conn.send(Box::new(login)).await.unwrap();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut tx = tx_clone;
        loop {
            match stream.next().await {
                None => {
                    break;
                }
                Some(Ok(resp)) => {
                    match tx.send(ClientMessage::Proto(resp)) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("{}",err);
                            break;
                        }
                    };
                }
                Some(Err(err)) => {
                    error!("{}",err);
                }
            }
        }
    });
    tokio::spawn(async move {
        let mut tx = tx.clone();
        loop {
            match tx.send(ClientMessage::Tick) {
                Ok(_) => {}
                Err(error) => {
                    error!("{}",error);
                    break;
                }
            };
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    client.start().await;
}