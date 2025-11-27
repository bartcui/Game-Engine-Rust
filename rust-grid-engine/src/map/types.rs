use crate::grid::GridCoord;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DoorSpec {
    pub x: i32,
    pub y: i32,
    pub locked: bool,
    pub key_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnemySpec {
    pub x: i32,
    pub y: i32,
    pub kind: String, 
}

#[derive(Debug, Clone, Deserialize)]
pub struct Level {
    pub name: Option<String>,
    pub width: i32,
    pub height: i32,

    pub seed: Option<u64>,

    pub player_start: GridCoord,
    pub walls: Vec<GridCoord>,
    pub goals: Vec<GridCoord>,

    #[serde(default)]
    pub traps: Vec<GridCoord>,

    #[serde(default)]
    pub doors: Vec<DoorSpec>,

    #[serde(default)]
    pub enemies: Vec<EnemySpec>,
}
