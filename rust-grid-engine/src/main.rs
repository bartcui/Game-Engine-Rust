use bevy::prelude::*;

mod components;
mod engine;
mod grid;
mod intents;
mod map;
mod pathfinding;
mod scenes;

use engine::EnginePlugin;
use scenes::ScenePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EnginePlugin, ScenePlugin))
        .run();
}
