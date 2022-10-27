use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext, Context};
use futures::SinkExt;
use log::{error, info};
use protobuf::MessageDyn;
use tokio::sync::Mutex;

use protocol::codec::MessageSink;

use crate::world::WorldActor;

pub struct PlayerActor {
    pub conn: Arc<Mutex<MessageSink>>,
    pub location: Location,
    pub player_id: i32,
    pub world_pid: Addr<WorldActor>,
}

impl PlayerActor {
    pub fn new(conn: MessageSink, world_pid: Addr<WorldActor>) -> Self {
        Self {
            player_id: 0,
            conn: Arc::new(Mutex::new(conn)),
            location: Location::default(),
            world_pid,
        }
    }
    pub fn send(&mut self, ctx: &mut Context<Self>, msg: Box<dyn MessageDyn>) {
        let conn = self.conn.clone();
        let player_id = self.player_id;
        let f = actix::fut::wrap_future(async move {
            match conn.lock().await.send(msg).await {
                Ok(_) => {}
                Err(err) => {
                    error!("player:{} send msg err:{}", player_id, err);
                }
            };
        });
        ctx.spawn(f);
    }
}

impl Actor for PlayerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("player:{} started", self.player_id);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("player:{} stopped", self.player_id);
    }
}

//每秒tick广播位置修正
#[derive(Default)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub activate: bool,
}
