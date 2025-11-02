use crate::components::*;
use crate::grid::{GridCoord, GridTransform};
use crate::intents::Intent;
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    InGame,
    GameOver,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .insert_resource(GridTransform::default())
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameScene::Menu), setup_menu)
            .add_systems(OnEnter(GameScene::InGame), setup_game)
            .add_systems(OnExit(GameScene::InGame), teardown_game)
            .add_systems(Update, sync_transforms.run_if(is_in_game_scene));
    }
}

pub fn is_in_game_scene(state: Res<State<GameScene>>) -> bool {
    *state.get() == GameScene::InGame
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_menu(mut next: ResMut<NextState<GameScene>>) {
    // Immediately jump into game for now
    next.set(GameScene::InGame);
}

fn setup_game(mut commands: Commands) {
    // Minimal test world: one player at (0,0)
    let p = GridCoord::new(0, 0);
    commands.spawn((
        Player,
        Actor,
        Position(p),
        PendingIntent(Intent::Wait),
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0),
            custom_size: Some(Vec2::splat(30.0)),
            ..Default::default()
        },
        Transform::from_translation(GridTransform::default().to_world(p)),
    ));
}

pub fn sync_transforms(
    grid_transform: Res<GridTransform>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    for (pos, mut transform) in &mut q {
        transform.translation = grid_transform.to_world(pos.0);
    }
}

fn teardown_game(mut commands: Commands, q: Query<Entity, With<Actor>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
