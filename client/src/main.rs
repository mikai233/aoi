use actix::{Actor, Addr, Message};
use futures::StreamExt;
use log::error;
use protobuf::{Enum, MessageDyn};
use rand::Rng;
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;

use protocol::codec::{MessageStream, ProtoCodec};
use protocol::mapper::kcp_config;

use crate::client::ClientActor;

mod client;
mod handler;

pub struct PoisonPill;

impl Message for PoisonPill {
    type Result = ();
}

pub struct Response(Box<dyn MessageDyn>);

impl Message for Response {
    type Result = ();
}

pub struct Request(Box<dyn MessageDyn>);

impl Message for Request {
    type Result = ();
}

pub struct Tick;

impl Message for Tick {
    type Result = ();
}

pub const PLAYER_COUNT: usize = 200;

pub const HORIZONTAL_BOUNDARY: f64 = 1000.;

pub const VERTICAL_BOUNDARY: f64 = 1000.;


#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    let mut clients = vec![];
    for _ in 0..PLAYER_COUNT {
        let c = actix_rt::spawn(start_client(addr));
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

fn start_receive_msg(mut stream: MessageStream, pid: Addr<ClientActor>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match stream.next().await {
                None => {
                    break;
                }
                Some(Ok(msg)) => {
                    pid.do_send(Response(msg));
                }
                Some(Err(e)) => {
                    error!("receive msg err:{}", e);
                    break;
                }
            };
        }
    })
}

async fn start_client(addr: &str) {
    let cfg = kcp_config();
    let stream = tokio_kcp::KcpStream::connect(&cfg, addr.parse().unwrap()).await.unwrap();

    let framed = Framed::new(stream, ProtoCodec::new(false));
    let (sink, mut stream) = framed.split();
    let client = ClientActor::new(sink);
    let pid = client.start();
    match start_receive_msg(stream, pid.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("{}",err);
        }
    };
}