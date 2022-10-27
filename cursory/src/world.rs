use std::collections::HashMap;

use actix::{Actor, Addr, Context};
use log::info;

use crate::player::PlayerActor;

pub struct WorldActor {
    pub world_id: i32,
    pub sessions: HashMap<i32, Addr<PlayerActor>>,
}

impl WorldActor {
    pub fn new() -> Self {
        Self {
            world_id: 0,
            sessions: HashMap::new(),
        }
    }
}

impl Actor for WorldActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("world:{} started",self.world_id);
    }
}