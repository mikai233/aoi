use std::time::Duration;

use actix::{ActorContext, AsyncContext, Handler};
use futures::SinkExt;
use log::info;
use protobuf::MessageField;
use rand::{random, Rng, thread_rng};

use protocol::test::{MoveCmd, MoveStartNotify, MoveStopNotify, Vector2};

use crate::{Cmd, PoisonPill, Request, Response, Tick};
use crate::client::ClientActor;

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
        info!("client:{} receive response:{}", self.player_id, msg.descriptor_dyn().name());
    }
}

impl Handler<Request> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Request, ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        info!("client:{} send request:{}", self.player_id, msg.descriptor_dyn().name());

        if let Some(req) = msg.clone_box().downcast_box::<MoveStartNotify>().ok() {
            self.state = req.cmd.unwrap();
            self.velocity = req.velocity;
            self.send(ctx, req);
        } else if let Some(mut req) = msg.clone_box().downcast_box::<MoveStopNotify>().ok() {
            let mut location = Vector2::new();
            location.x = self.x;
            location.y = self.y;
            self.state = MoveCmd::Idle;
            req.location = MessageField::some(location);
            self.send(ctx, req);
        } else {
            self.send(ctx, msg);
        }
    }
}

impl Handler<Tick> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Tick, ctx: &mut Self::Context) -> Self::Result {
        ctx.notify_later(Tick, Duration::from_millis(100));
        match self.state {
            MoveCmd::Idle => {}
            MoveCmd::MoveLeft => {
                self.x -= self.velocity;
            }
            MoveCmd::MoveRight => {
                self.x += self.velocity;
            }
            MoveCmd::MoveUp => {
                self.y += self.velocity;
            }
            MoveCmd::MoveDown => {
                self.y -= self.velocity;
            }
            MoveCmd::MoveLeftUp => {}
            MoveCmd::MoveLeftDown => {}
            MoveCmd::MoveRightUp => {}
            MoveCmd::MoveRightDown => {}
            MoveCmd::Jump => {}
        };
        let o = thread_rng().gen_ratio(1, 3);
        if o {
            random()
        }
    }
}

impl Handler<Cmd> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: Cmd, ctx: &mut Self::Context) -> Self::Result {
        info!("cmd:{}",msg.0);
        if msg.0 == "move start".to_string() {
            self.move_cmd(ctx, MoveCmd::MoveUp)
        }
        if msg.0 == "move stop".to_string() {}
    }
}
