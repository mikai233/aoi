use actix::{Actor, Addr};
use futures::StreamExt;
use log::{error, info};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use protocol::codec::{MessageStream, ProtoCodec};

use crate::message::{PlayerProtoMessage, PoisonPill};
use crate::player::PlayerActor;
use crate::world::WorldActor;

pub async fn new_client(stream: TcpStream, world: Addr<WorldActor>) -> anyhow::Result<()> {
    let framed = Framed::new(stream, ProtoCodec::new(true));
    let (sink, stream) = framed.split();
    let player = PlayerActor::new(sink, world);
    let pid = player.start();
    tokio::spawn(async move { receive_msg(stream, pid).await });
    Ok(())
}

async fn receive_msg(mut stream: MessageStream, pid: Addr<PlayerActor>) {
    loop {
        match stream.next().await {
            None => {
                info!("stream closed");
                break;
            }
            Some(Ok(msg)) => {
                pid.do_send(PlayerProtoMessage(msg));
            }
            Some(Err(e)) => {
                error!("receive msg err:{}", e);
                break;
            }
        }
    }
}
