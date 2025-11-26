use bevy::prelude::*;
pub mod types;
pub mod occupancy;
pub use types::*;   
pub use occupancy::OccupancyIndex;     
use crate::components::{Position, Blocking, Actor};

#[derive(Resource)]
pub struct GridTransform {
    pub tile_size: f32,
    pub origin: Vec2,
}

impl Default for GridTransform {
    fn default() -> Self {
        Self {
            tile_size: 32.0,
            origin: Vec2::new(0.0, 0.0),
        }
    }
}

impl GridTransform {
    pub fn to_world(&self, coord: GridCoord) -> Vec3 {
        Vec3::new(
            self.origin.x + coord.x as f32 * self.tile_size,
            self.origin.y + coord.y as f32 * self.tile_size,
            0.0,
        )
    }

    pub fn to_grid(&self, w: Vec2) -> GridCoord {
        let x = ((w.x - self.origin.x) / self.tile_size).floor() as i32;
        let y = ((w.y - self.origin.y) / self.tile_size).floor() as i32;
        GridCoord { x, y }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Layer {
    Terrain,
    Blockers,
    Actors,
    Items,
}

// 4 neighbours
pub fn neighbours_4(c: GridCoord) -> [GridCoord; 4] {
    [
        GridCoord::new(c.x + 1, c.y),
        GridCoord::new(c.x - 1, c.y),
        GridCoord::new(c.x, c.y + 1),
        GridCoord::new(c.x, c.y - 1),
    ]
}

// 8 neighbours for diagonals 
pub fn neighbours_8(c: GridCoord) -> [GridCoord; 8] {
    [
        GridCoord::new(c.x + 1, c.y),
        GridCoord::new(c.x - 1, c.y),
        GridCoord::new(c.x, c.y + 1),
        GridCoord::new(c.x, c.y - 1),
        GridCoord::new(c.x + 1, c.y + 1),
        GridCoord::new(c.x - 1, c.y + 1),
        GridCoord::new(c.x + 1, c.y - 1),
        GridCoord::new(c.x - 1, c.y - 1),
    ]
}

pub fn in_bounds(c: GridCoord, width: i32, height: i32) -> bool {
    c.x >= 0 && c.x < width && c.y >= 0 && c.y < height
}

// ECS system to rebuild the index each frame / turn
pub fn rebuild_occupancy(
    mut occ: ResMut<OccupancyIndex>,
    q: Query<(Entity, &Position, Option<&Blocking>, Option<&Actor>)>,
) {
    occ.clear();

    for (entity, pos, blocking, actor) in &q {
        // Put blocking things into Blockers layer
        if blocking.is_some() {
            occ.insert(Layer::Blockers, pos.0, entity);
        }
        // Put actors into Actors layer
        if actor.is_some() {
            occ.insert(Layer::Actors, pos.0, entity);
        }
    }
}