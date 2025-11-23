use bevy::prelude::*;
pub mod types;
pub mod occupancy;
pub use types::*;        

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
}
