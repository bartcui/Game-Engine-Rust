use bevy::prelude::*;

use rust_grid_engine::engine::EnginePlugin;
use rust_grid_engine::scenes::ScenePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EnginePlugin, ScenePlugin))
        .run();
}
