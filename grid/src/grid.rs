use std::collections::HashMap;

use crate::player::State;
use crate::world::L;

#[derive(Debug, Default, Clone)]
pub struct Grid {
    pub players: HashMap<i32, State>,
}

pub fn calculate_grid_id(x: f32, y: f32) -> (i32, i32) {
    let x_n = (x / L as f32) as i32;
    let y_n = (y / L as f32) as i32;
    return (x_n, y_n);
}

pub fn is_player_grid_change(pre_location: (f32, f32), curr_location: (f32, f32)) -> bool {
    let (p_n_x, p_n_y) = calculate_grid_id(pre_location.0, pre_location.1);
    let (c_n_x, c_n_y) = calculate_grid_id(curr_location.0, curr_location.1);
    if p_n_x != c_n_x || p_n_y != c_n_y {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::grid::calculate_grid_id;

    #[test]
    fn test_grid() {
        let x = 1.;
        let y = 1.;
        let (x_n, y_n) = calculate_grid_id(x, y);
        println!("x:{} x_n:{}|y:{} y_n:{}", x, x_n, y, y_n);
        let x = 20.;
        let y = 20.;
        let (x_n, y_n) = calculate_grid_id(x, y);
        println!("x:{} x_n:{}|y:{} y_n:{}", x, x_n, y, y_n);
    }
}
