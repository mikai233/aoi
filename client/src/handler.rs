use actix::{ActorContext, Handler};
use futures::SinkExt;
use log::info;

use crate::client::ClientActor;
use crate::{PoisonPill, Request, Response};

impl Handler<PoisonPill> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: PoisonPill, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<Response> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Response, ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        info!("client:{} receive response:{}", self.player_id, msg);
    }
}

impl Handler<Request> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Request, ctx: &mut Self::Context) -> Self::Result {
        info!("client:{} send request:{}", self.player_id, msg.0);
        self.send(ctx, msg.0);
    }
}
