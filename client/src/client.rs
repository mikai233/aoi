use std::sync::Arc;

use actix::{Actor, AsyncContext, Context};
use futures::SinkExt;
use log::{error, info};
use protobuf::{EnumOrUnknown, MessageDyn};
use rand::Rng;
use tokio::sync::Mutex;

use protocol::codec::MessageSink;
use protocol::test::{MoveCmd, MoveStartNotify, MoveStopNotify};

use crate::Tick;

pub struct ClientActor {
    pub player_id: i32,
    pub conn: Arc<Mutex<MessageSink>>,
    pub x: i32,
    pub y: i32,
    pub velocity: i32,
    pub state: MoveCmd,
}

impl ClientActor {
    pub fn new(conn: MessageSink) -> Self {
        Self {
            player_id: 0,
            conn: Arc::new(Mutex::new(conn)),
            x: 0,
            y: 0,
            velocity: 0,
            state: MoveCmd::Idle,
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
    pub fn move_cmd(&mut self, ctx: &mut Context<Self>, cmd: MoveCmd) {
        match self.state {
            MoveCmd::Idle => {}
            MoveCmd::MoveLeft |
            MoveCmd::MoveRight |
            MoveCmd::MoveUp |
            MoveCmd::MoveDown => {
                self.send(ctx, Box::new(MoveStopNotify::new()));
            }
            MoveCmd::MoveLeftUp => {}
            MoveCmd::MoveLeftDown => {}
            MoveCmd::MoveRightUp => {}
            MoveCmd::MoveRightDown => {}
            MoveCmd::Jump => {}
        };
        self.state = cmd;
        let mut req = MoveStartNotify::new();
        req.cmd = EnumOrUnknown::new(cmd);
        req.velocity = self.velocity;
        self.send(ctx, Box::new(MoveStartNotify::new()))
    }

    pub fn random_move(&mut self) {
        let mut rng = rand::thread_rng();
        let velocity = rand::thread_rng().gen_range(1..10);
        let v = vec![MoveCmd::MoveUp, MoveCmd::MoveDown, MoveCmd::MoveLeft, MoveCmd::MoveRight];
        let which = rng.gen_range(0..4);
        let move_cmd = v[which];
        self.state = move_cmd;
        self.velocity = velocity;
    }
}

impl Actor for ClientActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("client actor:{} started", self.player_id);
        ctx.notify(Tick);
    }
}
