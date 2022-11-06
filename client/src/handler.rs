use std::time::Duration;

use actix::{ActorContext, AsyncContext, Handler};
use futures::SinkExt;
use log::info;
use rand::{Rng, thread_rng};

use crate::{PoisonPill, Request, Response, Tick};
use crate::client::ClientActor;

impl Handler<PoisonPill> for ClientActor {
    type Result = ();

    fn handle(&mut self, _: PoisonPill, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<Response> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Response, ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        // info!("client:{} receive response:{}", self.player_id, msg.descriptor_dyn().name());
    }
}

impl Handler<Request> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Request, ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        let desc = msg.descriptor_dyn();
        let msg_name = desc.name();
        info!("client:{} send request:{}", self.player_id, msg_name);
        self.send(ctx, msg);
    }
}

impl Handler<Tick> for ClientActor {
    type Result = ();

    fn handle(&mut self, _: Tick, ctx: &mut Self::Context) -> Self::Result {
        ctx.notify_later(Tick, Duration::from_millis(100));
        let current_state = self.state;
        let change_state = thread_rng().gen_ratio(1, 3);
        if change_state {
            let new_state = ClientActor::random_state();
            if new_state != current_state {
                self.state = new_state;
                self.notify_and_move(ctx);
            }
        }
    }
}
