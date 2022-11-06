use std::sync::Arc;

use actix::{Actor, AsyncContext, Context};
use futures::SinkExt;
use futures::stream::SplitSink;
use log::{error, info};
use protobuf::{Enum, EnumOrUnknown, MessageDyn, MessageField};
use rand::{random, Rng};
use tokio::sync::Mutex;
use tokio_kcp::KcpStream;
use tokio_util::codec::Framed;

use protocol::codec::{MessageSink, ProtoCodec};
use protocol::test::{LoginReq, PlayerMoveNotify, State, Vector2};

use crate::{HORIZONTAL_BOUNDARY, Tick, VERTICAL_BOUNDARY};

pub struct ClientActor {
    pub player_id: i32,
    pub conn: Arc<Mutex<MessageSink>>,
    pub x: f64,
    pub y: f64,
    pub velocity: f64,
    pub state: State,
}

impl ClientActor {
    pub fn new(conn: SplitSink<Framed<KcpStream, ProtoCodec>, Box<dyn MessageDyn>>) -> Self {
        Self {
            player_id: 0,
            conn: Arc::new(Mutex::new(conn)),
            x: 0.,
            y: 0.,
            velocity: 10.,
            state: State::Idle,
        }
    }
    pub fn send(&mut self, ctx: &mut Context<Self>, msg: Box<dyn MessageDyn>) {
        let conn = self.conn.clone();
        let player_id = self.player_id;
        let f = actix::fut::wrap_future(async move {
            match conn.lock().await.send(msg).await {
                Ok(_) => {}
                Err(err) => {
                    error!("client:{} send msg err:{}", player_id, err);
                }
            };
        });
        ctx.spawn(f);
    }

    pub fn random_state() -> State {
        let mut rng = rand::thread_rng();
        let v = State::VALUES;
        let which = rng.gen_range(0..State::VALUES.len());
        let move_state = v[which];
        move_state
    }

    pub fn notify_and_move(&mut self, ctx: &mut Context<ClientActor>) {
        let mut notify = PlayerMoveNotify::new();
        notify.player_id = self.player_id;
        notify.state = EnumOrUnknown::new(self.state.clone());
        let mut v = Vector2::new();
        v.x = self.x;
        v.y = self.y;
        notify.location = MessageField::some(v);
        self.send(ctx, Box::new(notify));

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
    }
}

impl Actor for ClientActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("client actor:{} started", self.player_id);
        let player_id = random();
        self.player_id = player_id;
        let mut login = LoginReq::new();
        login.player_id = player_id;
        self.send(ctx, Box::new(login));
        ctx.notify(Tick);
    }
}
