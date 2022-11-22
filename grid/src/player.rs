use std::net::SocketAddr;
use std::ops::Range;

use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use log::{error, info};
use rand::{Rng, thread_rng};
use tokio::task::JoinHandle;
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::test::{Color, PlayerState};

use crate::message::{PlayerMessageReceiver, PlayerMessageSender, PlayerMessageWrap, ProtoMessage, ProtoMessageReceiver, ProtoMessageSender, WorldMessageSender};

pub struct Player {
    pub player_id: i32,
    pub addr: SocketAddr,
    pub player_sender: PlayerMessageSender,
    pub proto_sender: ProtoMessageSender,
    pub world_sender: WorldMessageSender,
    pub state: State,
}

impl Player {
    pub fn new(addr: SocketAddr, player_sender: PlayerMessageSender, proto_sender: ProtoMessageSender, world_sender: WorldMessageSender) -> Self {
        Self {
            player_id: 0,
            addr,
            player_sender,
            proto_sender,
            world_sender,
            state: State::default(),
        }
    }

    pub async fn handle_req(&mut self, msg: ProtoMessage) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn handle_player_msg(&mut self, msg: PlayerMessageWrap) -> anyhow::Result<()> {
        Ok(())
    }


    pub fn start_receive_msg(player: Player, mut read: SplitStream<Framed<KcpStream, ProtoCodec>>, mut player_receiver: PlayerMessageReceiver) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut player = player;
            loop {
                tokio::select! {
                    Some(Ok(request)) = read.next() => {
                        match player.handle_req(request).await {
                            Ok(_) => {}
                            Err(error) => {
                                error!("player {} handle msg error {}",player.player_id,error);
                            }
                        };
                    }
                    Some(message) = player_receiver.recv() => {
                        match player.handle_player_msg(message).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("player {} handle player msg err {}",player.player_id,err);
                            }
                        };
                    }
                    else => {
                        info!("player {} stopped",player.player_id);
                        break;
                    }
                }
            }
        })
    }

    pub fn start_write_msg(mut proto_receiver: ProtoMessageReceiver, mut write: SplitSink<Framed<KcpStream, ProtoCodec>, ProtoMessage>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match proto_receiver.recv().await {
                    None => {
                        info!("all proto sender dropped, close the receiver");
                        break;
                    }
                    Some(msg) => {
                        match write.send(msg).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("send proto message err:{}, close the receiver",err);
                                break;
                            }
                        };
                    }
                }
            }
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct State {
    pub player_state: PlayerState,
    pub color: Color,
}

pub fn random_color() -> Color {
    let range: Range<f32> = 0.0..255.;
    let mut thread_rng = thread_rng();
    let mut color = Color::new();
    color.r = thread_rng.gen_range(range.clone());
    color.g = thread_rng.gen_range(range.clone());
    color.b = thread_rng.gen_range(range.clone());
    color
}
