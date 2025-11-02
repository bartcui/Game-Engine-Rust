use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Logical integer grid coord (x,y)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}
impl GridCoord {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Direction in 4-neighbour grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn step(self, p: GridCoord) -> GridCoord {
        match self {
            Dir::Up => GridCoord { x: p.x, y: p.y + 1 },
            Dir::Down => GridCoord { x: p.x, y: p.y - 1 },
            Dir::Left => GridCoord { x: p.x - 1, y: p.y },
            Dir::Right => GridCoord { x: p.x + 1, y: p.y },
        }
    }
}

/// Mapping between grid and world space
#[derive(Resource, Debug, Clone, Copy)]
pub struct GridTransform {
    pub tile_size: Vec2,
    pub origin: Vec2, // world coordinate of (0,0)
}
impl Default for GridTransform {
    fn default() -> Self {
        Self {
            tile_size: Vec2::splat(32.0),
            origin: Vec2::ZERO,
        }
    }
}
//to_world and to_grid methods
impl GridTransform {
    pub fn to_world(&self, c: GridCoord) -> Vec3 {
        Vec3::new(
            self.origin.x + c.x as f32 * self.tile_size.x,
            self.origin.y + c.y as f32 * self.tile_size.y,
            0.0,
        )
    }
    pub fn to_grid(&self, w: Vec2) -> GridCoord {
        let x = ((w.x - self.origin.x) / self.tile_size.x).floor() as i32;
        let y = ((w.y - self.origin.y) / self.tile_size.y).floor() as i32;
        GridCoord { x, y }
    }
}

/// Distinct layers for occupancy / collision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Terrain,
    Units,
    Items,
}
