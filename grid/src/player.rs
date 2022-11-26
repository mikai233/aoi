use std::net::SocketAddr;
use std::ops::{Not, Range};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use log::{debug, error, info};
use protobuf::Message;
use rand::{Rng, thread_rng};
use tokio::task::JoinHandle;
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::test::{Color, LoginReq, PlayerMoveNotify, PlayerState};

use crate::event::ReceiveTimeoutEvent;
use crate::message::{PlayerMessage, PlayerMessageReceiver, PlayerMessageSender, PlayerMessageWrap, ProtoMessage, ProtoMessageReceiver, ProtoMessageSender, WorldMessageSender};
use crate::player_handler::{handle_event, handle_login_req, handle_move_req, handle_world_kick_out};
use crate::tick::Ticker;

#[derive(Debug, Clone)]
pub struct PlayerSender {
    pub player: PlayerMessageSender,
    pub proto: ProtoMessageSender,
}

pub struct Player {
    pub player_id: i32,
    pub addr: SocketAddr,
    pub player_sender: PlayerMessageSender,
    pub proto_sender: ProtoMessageSender,
    pub world_sender: WorldMessageSender,
    pub state: State,
    pub write_handle: Option<JoinHandle<()>>,
    pub stooped: bool,
    pub ticker: Ticker,
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
            write_handle: None,
            stooped: false,
            ticker: Ticker::new(),
        }
    }

    pub fn stop(&mut self) {
        info!("player {} stop",self.player_id);
        self.stooped = true;
        if let Some(w) = &self.write_handle {
            w.abort();
            info!("abort player {} write handle",self.player_id);
        }
    }

    pub async fn handle_req(&mut self, msg: ProtoMessage) -> anyhow::Result<()> {
        self.ticker.cancel(ReceiveTimeoutEvent.to_string());
        let desc = msg.descriptor_dyn();
        let msg_name = desc.name();
        if msg_name == LoginReq::NAME {
            handle_login_req(self, msg).await?;
        } else if msg_name == PlayerMoveNotify::NAME {
            handle_move_req(self, msg).await?;
        }
        Ok(())
    }

    pub async fn handle_player_msg(&mut self, msg: PlayerMessageWrap) -> anyhow::Result<()> {
        self.ticker.cancel(ReceiveTimeoutEvent.to_string());
        let world_id = msg.world_id;
        match msg.message {
            PlayerMessage::KickOut(reason) => { handle_world_kick_out(self, world_id, reason).await?; }
            PlayerMessage::Event(_) => {}
        }
        Ok(())
    }


    pub fn start_receive_msg(player: Player, mut read: SplitStream<Framed<KcpStream, ProtoCodec>>, mut player_receiver: PlayerMessageReceiver) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut player = player;
            while player.stooped.not() {
                player.ticker.schedule_once(Duration::from_secs(10), ReceiveTimeoutEvent.to_string(), Box::new(ReceiveTimeoutEvent));
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
                    Some(event) = player.ticker.handle_event() => {
                        match handle_event(&mut player,event).await {
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
            debug!("write task done")
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
