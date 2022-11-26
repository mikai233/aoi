use std::collections::HashMap;

use protocol::test::PlayerState;

use crate::player::State;
use crate::world::{H, V};

#[derive(Debug, Default, Clone)]
pub struct Grid {
    pub x: f32,
    pub y: f32,
    pub players: HashMap<i32, State>,
}

pub fn calculate_grid_id(x: f32, y: f32) -> (usize, usize) {
    let x_n = (x / H as f32) as usize;
    let y_n = (y / V as f32) as usize;
    return (x_n, y_n);
}

#[cfg(test)]
mod test {}
