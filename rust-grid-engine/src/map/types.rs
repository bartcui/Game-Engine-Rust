use crate::grid::GridCoord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub width: i32,
    pub height: i32,
    pub player_start: GridCoord,
    pub walls: Vec<GridCoord>,
    pub goals: Vec<GridCoord>,
}
