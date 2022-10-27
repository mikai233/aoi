use actix::{Actor, Addr, Message};
use futures::StreamExt;
use log::{error, info};
use protobuf::MessageDyn;
use tokio::io;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed, FramedRead};

use protocol::codec::{MessageStream, ProtoCodec};
use protocol::test::LoginReq;

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

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();
    let addr = "127.0.0.1:4895";
    let stdin = FramedRead::new(io::stdin(), BytesCodec::new());
    let mut stdin = stdin.map(|i| i.map(|bytes| bytes.freeze()));
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, ProtoCodec::new(false));
    let (sink, mut stream) = framed.split();
    let client = ClientActor::new(sink);
    let pid = client.start();
    start_receive_msg(stream, pid.clone());
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("signal ctrl c, close client");
                break;
            }
            input = stdin.next() => match input {
                Some(Ok(input)) => {
                    process_client_input(input, &pid).await?;
                }
                Some(Err(e)) => {
                    error!("input err:{}", e);
                }
                None => break,
            }
        }
    }
    Ok(())
}

fn start_receive_msg(mut stream: MessageStream, pid: Addr<ClientActor>) {
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
    });
}

async fn process_client_input(input: bytes::Bytes, pid: &Addr<ClientActor>) -> anyhow::Result<()> {
    let cmd = String::from_utf8_lossy(input.as_ref())
        .trim_end()
        .to_string();
    info!("cmd:{}", cmd);
    if cmd.starts_with("login") {
        let cmd = cmd.splitn(2, ' ').collect::<Vec<&str>>();
        let player_id: i32 = cmd[1].parse()?;
        let mut login = LoginReq::new();
        login.player_id = player_id;
        pid.do_send(Request(Box::new(login)));
    }
    Ok(())
}
