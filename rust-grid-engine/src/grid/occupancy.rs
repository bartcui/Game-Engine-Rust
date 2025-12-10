use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;

use super::{GridCoord, Layer};

#[derive(Resource, Default, Clone)]
pub struct OccupancyIndex {
    // For each layer, a map of grid cell to entities present.
    map: HashMap<Layer, HashMap<GridCoord, SmallVec<[Entity; 4]>>>,
}

impl OccupancyIndex {
    pub fn clear(&mut self) {
        self.map.clear();
    }
    pub fn insert(&mut self, layer: Layer, coord: GridCoord, e: Entity) {
        use std::collections::hash_map::Entry;
        let layer_map = self.map.entry(layer).or_default();
        match layer_map.entry(coord) {
            Entry::Vacant(v) => {
                v.insert(smallvec![e]);
            }
            Entry::Occupied(mut o) => o.get_mut().push(e),
        }
    }
    pub fn at(&self, layer: Layer, coord: GridCoord) -> &[Entity] {
        // const/STATIC-friendly empty slice
        static EMPTY: [Entity; 0] = [];
        self.map
            .get(&layer)
            .and_then(|m| m.get(&coord))
            .map(|v| v.as_slice()) // SmallVec -> &[Entity]
            .unwrap_or(&EMPTY)
    }
    pub fn is_occupied(&self, layer: Layer, coord: GridCoord) -> bool {
        !self.at(layer, coord).is_empty()
    }
}
