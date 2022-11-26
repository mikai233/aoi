use std::collections::{HashMap, HashSet};
use std::ops::Not;

use log::{error, info, warn};
use protobuf::{MessageDyn, MessageField};

use protocol::test::{PlayerState, SCPlayerEnterNotify, SCPlayerLeaveNotify, SCPlayerMoveNotify};

use crate::grid::{calculate_grid_id, Grid};
use crate::message::{KickOutReason, PlayerLoginData, PlayerMessage, PlayerMessageWrap, WorldMessage, WorldMessageSender, WorldMessageWrap};
use crate::player::{PlayerSender, State};
use crate::world_handler::{handle_player_login, handle_player_move};

pub const H: usize = 200;
pub const V: usize = 200;
pub const L: usize = 20;

///
///
///           player h_side h_side h_side
///           v_side
///           v_side
///           v_side
///
pub const AOI_H_SIDE: usize = 10;
pub const AOI_V_SIDE: usize = 10;

pub struct World {
    pub world_id: i32,
    pub sessions: HashMap<i32, PlayerSender>,
    pub player_grid: HashMap<i32, (i32, i32)>,
    pub grids: HashMap<i32, HashMap<i32, Grid>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            world_id: 0,
            sessions: HashMap::new(),
            player_grid: HashMap::new(),
            grids: HashMap::new(),
        }
    }

    pub async fn handle_world_msg(&mut self, msg: WorldMessageWrap) -> anyhow::Result<()> {
        let player_id = msg.player_id;
        match msg.message {
            WorldMessage::PlayerLogin(data) => {
                handle_player_login(self, player_id, data).await?;
            }
            WorldMessage::PlayerLogout => {}
            WorldMessage::PlayerMove(data) => {
                handle_player_move(self, player_id, data).await?;
            }
            WorldMessage::Proto(_) => {}
        }
        Ok(())
    }

    /// broadcast msg to current player's aoi view players
    pub fn broadcast_msg_to_player_aoi(&mut self, current_player: i32, msg: Box<dyn MessageDyn>, include_self: bool) {
        let (n_x, n_y) = self.player_grid[&current_player];
        let aoi_grids = self.get_player_aoi_view(n_x, n_y);
        let mut aoi_players = HashSet::new();
        for (_, grid) in aoi_grids {
            let players: Vec<i32> = grid.players.keys().map(|&k| { k }).collect();
            aoi_players.extend(players)
        }
        if include_self.not() {
            aoi_players.remove(&current_player);
        }
        self.broadcast_msg(Vec::from_iter(aoi_players.into_iter()), msg);
    }

    pub fn broadcast_msg_to_grid(&mut self, grid: &Grid, msg: Box<dyn MessageDyn>) {
        let players: Vec<i32> = grid.players.keys().map(|&k| { k }).collect();
        self.broadcast_msg(players, msg);
    }

    pub fn broadcast_msg(&mut self, players: Vec<i32>, msg: Box<dyn MessageDyn>) {
        let mut remove_players = vec![];
        for player_id in players {
            let sender = &self.sessions[&player_id];
            if let Some(err) = sender.proto.send(msg.clone()).err() {
                warn!("broadcast message to player {} err {}, player session will be remove",player_id,err);
                remove_players.push(player_id);
            }
        }
        self.remove_players(remove_players);
    }

    pub fn remove_players(&mut self, players: Vec<i32>) {
        for player_id in players {
            if let Some(sender) = self.sessions.remove(&player_id) {
                let _ = sender.player.send(PlayerMessageWrap::new(self.world_id, PlayerMessage::KickOut(KickOutReason::MultiLogin("other player login with same account".to_string()))));
                info!("player {} session removed from world {}",player_id,self.world_id);
            }
            if let Some(grid) = self.player_grid.remove(&player_id) {
                self.remove_player_from_grid_by_grid_id(player_id, grid.0, grid.1);
            }
        }
    }

    pub fn add_player(&mut self, player_id: i32, player_login_data: PlayerLoginData) {
        self.remove_players(vec![player_id]);
        self.sessions.insert(player_id, player_login_data.sender);
        self.add_player_to_grid(player_id, player_login_data.state)
    }

    pub fn search_grid_by_location(&mut self, x: f32, y: f32) -> Option<&mut Grid> {
        let n_x = (x / L as f32) as i32;
        let n_y = (y / L as f32) as i32;
        if let Some(column) = self.grids.get_mut(&n_x) {
            column.get_mut(&n_y)
        } else {
            None
        }
    }

    pub fn search_grid_by_grid_id_mut(&mut self, n_x: i32, n_y: i32) -> Option<&mut Grid> {
        if let Some(column) = self.grids.get_mut(&n_x) {
            column.get_mut(&n_y)
        } else {
            None
        }
    }

    pub fn search_grid_by_grid_id(&self, n_x: i32, n_y: i32) -> Option<&Grid> {
        if let Some(column) = self.grids.get(&n_x) {
            column.get(&n_y)
        } else {
            None
        }
    }

    pub fn add_player_to_grid(&mut self, player_id: i32, mut state: State) {
        let player_state = &state.player_state;
        let x = player_state.x;
        let y = player_state.y;
        let (n_x, n_y) = calculate_grid_id(x, y);
        self.player_grid.insert(player_id, (n_x, n_y));
        if let Some(grid) = self.search_grid_by_location(x, y) {
            grid.players.insert(player_id, state);
        } else {
            let column = self.grids.entry(n_x).or_insert(Default::default());
            let grid = column.entry(n_y).or_insert(Default::default());
            grid.players.insert(player_id, state);
        }
        let mut player_enter_notify = SCPlayerEnterNotify::new();
        self.broadcast_msg_to_player_aoi(player_id, Box::new(player_enter_notify), false);
    }

    pub fn remove_player_from_grid_by_grid_id(&mut self, player_id: i32, n_x: i32, n_y: i32) {
        if let Some(grid) = self.search_grid_by_grid_id_mut(n_x, n_y) {
            let players = &mut grid.players;
            if players.remove(&player_id).is_some() {
                let mut player_leave_notify = SCPlayerLeaveNotify::new();
                player_leave_notify.player_id = player_id;
                self.broadcast_msg_to_player_aoi(player_id, Box::new(player_leave_notify), false);
            }
        }
    }

    pub fn get_player_aoi_view(&self, n_x: i32, n_y: i32) -> HashMap<(i32, i32), &Grid> {
        let mut aoi_grid_id = vec![];
        let mut aoi_grids = HashMap::new();
        let center = self.search_grid_by_grid_id(n_x, n_y).expect(&*format!("aoi grid:({},{}) not found", n_x, n_y));
        aoi_grids.insert((n_x, n_y), center);
        //left
        let mut left_tmp = n_x;
        for i in 1..=AOI_H_SIDE {
            left_tmp -= L as i32;
            aoi_grid_id.push((left_tmp, n_y));
        }
        //right
        let mut right_tmp = n_x;
        for i in 1..=AOI_H_SIDE {
            right_tmp += L as i32;
            aoi_grid_id.push((right_tmp, n_y));
        }
        //up
        let mut up_tmp = n_y;
        for i in 1..=AOI_V_SIDE {
            up_tmp -= L as i32;
            aoi_grid_id.push((n_x, up_tmp));
        }
        //up
        let mut down_tmp = n_y;
        for i in 1..=AOI_V_SIDE {
            down_tmp += L as i32;
            aoi_grid_id.push((n_x, down_tmp));
        }
        for (n_x, n_y) in aoi_grid_id {
            if let Some(grid) = self.search_grid_by_grid_id(n_x, n_y) {
                aoi_grids.insert((n_x, n_y), grid);
            }
        }
        aoi_grids
    }

    pub fn is_player_grid_change(&self, pre_location: (f32, f32), curr_location: (f32, f32)) -> bool {
        let (p_n_x, p_n_y) = calculate_grid_id(pre_location.0, pre_location.1);
        let (c_n_x, c_n_y) = calculate_grid_id(curr_location.0, curr_location.1);
        if p_n_x != c_n_x || p_n_y != c_n_y {
            true
        } else {
            false
        }
    }

    pub fn move_player(&mut self, player_id: i32, new_player_state: PlayerState) {
        let current_x = new_player_state.x;
        let current_y = new_player_state.y;
        let (n_x, n_y) = self.player_grid[&player_id];
        let grid = self.search_grid_by_grid_id_mut(n_x, n_y).expect(&*format!("the player:{} player grid:({},{}) not found", player_id, n_x, n_y));
        let state = &grid.players[&player_id];
        let previous_x = state.player_state.x;
        let previous_y = state.player_state.y;
        let mut player_move_notify = SCPlayerMoveNotify::new();
        player_move_notify.player_id = player_id;
        player_move_notify.state = MessageField::some(new_player_state);
        if self.is_player_grid_change((previous_x, previous_y), (current_x, current_y)) {
            //remove player form old grid and join the new grid
            let previous_grid = calculate_grid_id(previous_x, previous_y);
            let previous_aoi_view = self.get_player_aoi_view(previous_grid.0, previous_grid.1);
            let current_grid = calculate_grid_id(previous_x, previous_y);
            let current_aoi_view = self.get_player_aoi_view(current_grid.0, current_grid.1);
            let mut leave_grids = vec![];
            let mut enter_grids = vec![];
            for (grid_id, g) in &previous_aoi_view {
                if current_aoi_view.contains_key(&grid_id).not() {
                    leave_grids.push(g);
                }
            }

            for (grid_id, g) in &current_aoi_view {
                if previous_aoi_view.contains_key(&grid_id).not() {
                    enter_grids.push(g);
                }
            }
        } else {
            //player grid not change, just notify all the aoi players
            self.broadcast_msg_to_player_aoi(player_id, Box::new(player_move_notify), true);
        }
    }
}

pub fn start_world() -> WorldMessageSender {
    let world = World::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WorldMessageWrap>();
    tokio::spawn(async move {
        let mut world = world;
        loop {
            match rx.recv().await {
                None => {
                    //world dont stop
                }
                Some(message) => {
                    match world.handle_world_msg(message).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("world {} handle message error {}",world.world_id,err);
                        }
                    }
                }
            };
        }
    });
    tx
}