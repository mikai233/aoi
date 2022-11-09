use futures::SinkExt;
use futures::stream::SplitSink;
use log::info;
use protobuf::{Enum, EnumOrUnknown, MessageDyn, MessageField};
use rand::{Rng, thread_rng};
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::test::{PlayerMoveNotify, State, Vector2};

use crate::{HORIZONTAL_BOUNDARY, VERTICAL_BOUNDARY};

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
    pub x: f64,
    pub y: f64,
    pub velocity: f64,
    pub state: State,
}

impl Client {
    pub fn new(conn: MessageSink, tx: Tx, rx: Rx) -> Self {
        Self {
            player_id: 0,
            conn,
            tx,
            rx,
            x: 0.,
            y: 0.,
            velocity: 10.,
            state: State::Idle,
        }
    }

    pub fn random_state() -> State {
        let mut rng = rand::thread_rng();
        let v = State::VALUES;
        let which = rng.gen_range(0..State::VALUES.len());
        let move_state = v[which];
        move_state
    }

    pub async fn notify_and_move(&mut self) -> anyhow::Result<()> {
        let mut notify = PlayerMoveNotify::new();
        notify.player_id = self.player_id;
        notify.state = EnumOrUnknown::new(self.state.clone());
        let mut v = Vector2::new();
        v.x = self.x;
        v.y = self.y;
        notify.location = MessageField::some(v);
        self.conn.send(Box::new(notify)).await?;

        match self.state {
            State::Idle => {}
            State::MoveLeft => {
                self.x -= self.velocity;
            }
            State::MoveRight => {
                self.x += self.velocity;
            }
            State::MoveUp => {
                self.y -= self.velocity;
            }
            State::MoveDown => {
                self.y += self.velocity;
            }
        }
        if self.x < -HORIZONTAL_BOUNDARY {
            self.x = -HORIZONTAL_BOUNDARY;
        }
        if self.x > HORIZONTAL_BOUNDARY {
            self.x = HORIZONTAL_BOUNDARY;
        }
        if self.y < -VERTICAL_BOUNDARY {
            self.y = -VERTICAL_BOUNDARY;
        }
        if self.y > VERTICAL_BOUNDARY {
            self.y = VERTICAL_BOUNDARY;
        }
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
                            info!("client:{} receive msg:{}=>{}",self.player_id,msg_name,resp);
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
        //todo 插值之后再处理
        // let current_state = self.state;
        // let change_state = thread_rng().gen_ratio(1, 10);
        // if change_state {
        //     let new_state = Client::random_state();
        //     if new_state != current_state {
        //         self.state = new_state;
        //         self.notify_and_move().await.unwrap();
        //     }
        // }
        self.notify_and_move().await.unwrap();
        if thread_rng().gen_ratio(1, 10) {
            self.state = Client::random_state();
        }
    }
}