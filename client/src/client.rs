use std::sync::Arc;

use actix::{Actor, AsyncContext, Context};
use futures::SinkExt;
use log::{error, info};
use protobuf::MessageDyn;
use tokio::sync::Mutex;

use protocol::codec::MessageSink;

pub struct ClientActor {
    pub player_id: i32,
    pub conn: Arc<Mutex<MessageSink>>,
}

impl ClientActor {
    pub fn new(conn: MessageSink) -> Self {
        Self {
            player_id: 0,
            conn: Arc::new(Mutex::new(conn)),
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
}

impl Actor for ClientActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("client actor:{} started", self.player_id);
    }
}
