use std::future::Future;

use futures::{SinkExt, StreamExt};
use log::{error, warn};
use protobuf::{Message, MessageDyn};
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::{ProtoCodec, ProtoCodecError};
use protocol::test::{LoginReq, PlayerMoveNotify};

use crate::message::{PlayerMessageWrap, WorldMessageWrap};
use crate::player::Player;
use crate::player_handler::{handle_login_req, handle_move_notify};

pub async fn new_client(stream: KcpStream, world_sender: tokio::sync::mpsc::UnboundedSender<WorldMessageWrap>) -> anyhow::Result<()> {
    let framed = Framed::new(stream, ProtoCodec::new(true));
    let (mut sink, mut stream) = framed.split();
    let (player_sender, mut player_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (proto_sender, mut proto_receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            match proto_receiver.recv().await {
                None => {}
                Some(msg) => {
                    match sink.send(msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("send msg to client err:{}",err);
                            break;
                        }
                    }
                }
            }
        }
    });
    tokio::spawn(async move {
        let mut player = Player::new(player_sender, proto_sender, world_sender);
        loop {
            tokio::select! {
                req = stream.next() => {
                    if let Some(req) = req {
                        handle_client_req(&mut player, req).await;
                    }else {
                        break;
                    }
                }
                player_msg = player_receiver.recv() => {
                    handle_player_msg(&mut player, player_msg).await;
                }
            }
        }
    });
    Ok(())
}

async fn handle_client_req(player: &mut Player, req: Result<Box<dyn MessageDyn>, ProtoCodecError>) {
    async fn handle_inner(player: &mut Player, req_name: &str, msg: Box<dyn MessageDyn>) -> anyhow::Result<()> {
        match req_name {
            LoginReq::NAME => handle_login_req(player, msg).await?,
            PlayerMoveNotify::NAME => handle_move_notify(player, msg).await?,
            _ => {
                warn!("unhanded msg:{}",req_name);
            }
        }
        Ok(())
    }
    match req {
        Ok(req) => {
            let desc = req.descriptor_dyn();
            let req_name = desc.name();
            match handle_inner(player, req_name, req).await {
                Ok(_) => {}
                Err(err) => {
                    warn!("player:{} handle msg err:{}",player.player_id,err);
                }
            }
        }
        Err(err) => {
            error!("player:{} handle client req err:{}",player.player_id,err)
        }
    }
}

async fn handle_player_msg(player: &mut Player, wrap: Option<PlayerMessageWrap>) {
    if let Some(wrap) = wrap {
        todo!()
    }
}