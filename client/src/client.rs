use std::collections::LinkedList;
use std::time::SystemTime;

use futures::SinkExt;
use futures::stream::SplitSink;
use log::{info, warn};
use protobuf::{MessageDyn, MessageField, MessageFull};
use rand::{Rng, thread_rng};
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::mapper::cast;
use protocol::test::{PlayerMoveNotify, PlayerState, SCPlayerMoveNotify};

use crate::TICK_DURATION;

const MOVE_SPEED: f32 = 20.;

const HORIZONTAL_BOUNDARY: f32 = 10.;

const VERTICAL_BOUNDARY: f32 = 10.;

type Tx = tokio::sync::mpsc::UnboundedSender<ClientMessage>;
type Rx = tokio::sync::mpsc::UnboundedReceiver<ClientMessage>;
type MessageSink = SplitSink<Framed<KcpStream, ProtoCodec>, Box<dyn MessageDyn>>;

pub enum ClientMessage {
    Proto(Box<dyn MessageDyn>),
    Tick,
}

pub struct Client {
    pub player_id: i32,
    pub conn: MessageSink,
    pub tx: Tx,
    pub rx: Rx,
    pub pending_states: LinkedList<PlayerState>,
    pub current_state: PlayerState,
}

impl Client {
    pub fn new(conn: MessageSink, tx: Tx, rx: Rx) -> Self {
        Self {
            player_id: 0,
            conn,
            tx,
            rx,
            pending_states: LinkedList::new(),
            current_state: PlayerState::new(),
        }
    }

    pub fn move_player(&mut self) -> anyhow::Result<()> {
        let delta_mills = TICK_DURATION.as_secs_f32();
        let move_delta = delta_mills as f32 * self.current_state.speed;
        self.current_state.x += move_delta;
        self.current_state.y += move_delta;
        Ok(())
    }

    pub async fn notify_server(&mut self, new_state: PlayerState) -> anyhow::Result<()> {
        let mut notify = PlayerMoveNotify::new();
        notify.state = MessageField::some(new_state);
        self.conn.send(Box::new(notify)).await?;
        Ok(())
    }

    pub async fn start(&mut self) {
        loop {
            match self.rx.recv().await {
                None => {
                    break;
                }
                Some(message) => {
                    match message {
                        ClientMessage::Proto(resp) => {
                            let desc = resp.descriptor_dyn();
                            let msg_name = desc.name();
                            info!("{} {}",msg_name,resp);
                            if msg_name == SCPlayerMoveNotify::descriptor().name() {
                                let notify = cast::<SCPlayerMoveNotify>(resp).unwrap();
                                self.handle_sc_player_move_notify(notify)
                            }
                        }
                        ClientMessage::Tick => {
                            self.handle_tick().await;
                        }
                    }
                }
            }
        }
    }

    async fn handle_tick(&mut self) {
        //通知服务端进行移动
        self.notify_server(self.current_state.clone()).await.unwrap();
        self.pending_states.push_back(self.current_state.clone());
        //移动
        self.move_player().unwrap();
        //改变速度
        if thread_rng().gen_ratio(1, 3) {
            let speed = random_speed();
            self.current_state.speed = speed;
        }
        //change rotation
        if thread_rng().gen_ratio(1, 3) {
            let rotation = random_rotation();
            self.current_state.rotation = rotation;
        }
    }

    fn handle_sc_player_move_notify(&mut self, notify: Box<SCPlayerMoveNotify>) {
        if notify.player_id == self.player_id {
            //服务器权威输入
            let authoritative_state = notify.state.unwrap();

            if let Some(pending_state) = self.pending_states.pop_front() {
                if authoritative_state != pending_state {
                    self.current_state = authoritative_state.clone();
                    self.pending_states.clear();
                    warn!("状态回滚:{}=>{}",authoritative_state,self.current_state);
                }
            }
        }
    }
}

pub fn get_system_time() -> u128 {
    return SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
}

pub fn random_speed() -> f32 {
    let lower: f32 = -10.;
    let higher: f32 = 10.;
    thread_rng().gen_range(lower..=higher)
}

pub fn random_rotation() -> f32 {
    thread_rng().gen_range(0.0..360.)
}