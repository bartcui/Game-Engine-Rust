use crate::components::*;
use crate::engine::TurnNumber;
use crate::grid::{GridCoord,GridTransform};
use crate::intents::Intent;
use crate::map::load_level_from_json;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextFont, TextColor};
use std::fs;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    InGame,
    GameOver,
}

#[derive(Component)]
pub struct TurnHudText;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .insert_resource(GridTransform::default())
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameScene::Menu), setup_menu)
            .add_systems(OnEnter(GameScene::InGame), (setup_game, setup_hud))
            .add_systems(OnExit(GameScene::InGame), teardown_game)
            .add_systems(
                Update,
                (
                    sync_transforms,
                    update_turn_hud,
                )
                    .run_if(is_in_game_scene),
            );
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

fn setup_game(
    mut commands: Commands,
    grid_tf: Res<GridTransform>,
    //mut turn_rng: ResMut<TurnRng>, // if use level.seed
) {
    // game file loaded here
    let bytes = fs::read("assets/levels/level2.json")
        .expect("failed to read assets/levels/level2.json");
    let level = load_level_from_json(&bytes).expect("invalid level JSON");

    // If the level JSON has a seed, use it to re-seed the RNG
    //if let Some(seed) = level.seed {
      //  turn_rng.0 = StdRng::seed_from_u64(seed);
    //}

    // player
    let p = level.player_start;
    commands.spawn((
        Player,
        Actor,
        Position(p),
        PendingIntent(Intent::Wait),
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0),
            custom_size: Some(Vec2::splat(grid_tf.tile_size)),
            ..Default::default()
        },
        Transform::from_translation(grid_tf.to_world(p)),
    ));

    // walls
    for w in level.walls {
        commands.spawn((
            Blocking,
            Position(w),
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::splat(grid_tf.tile_size)),
                ..Default::default()
            },
            Transform::from_translation(grid_tf.to_world(w)),
        ));
    }

    // goals
    for g in level.goals {
        commands.spawn((
            Goal,
            Position(g),
            Sprite {
                color: Color::srgb(0.1, 0.9, 0.1),
                custom_size: Some(Vec2::splat(grid_tf.tile_size)),
                ..Default::default()
            },
            Transform::from_translation(grid_tf.to_world(g)),
        ));
    }

    // traps
    for t in level.traps {
        commands.spawn((
            Trap,
            Position(t),
            Sprite {
                color: Color::srgb(0.9, 0.2, 0.2),
                custom_size: Some(Vec2::splat(grid_tf.tile_size)),
                ..Default::default()
            },
            Transform::from_translation(grid_tf.to_world(t)),
        ));
    }

    // doors
    for d in level.doors {
        let coord = GridCoord::new(d.x, d.y);
        commands.spawn((
            Door,       
            Blocking,   
            Position(coord),
            Sprite {
                color: if d.locked {
                    Color::srgb(0.7, 0.5, 0.2)
                } else {
                    Color::srgb(0.9, 0.8, 0.3)
                },
                custom_size: Some(Vec2::splat(grid_tf.tile_size)),
                ..Default::default()
            },
            Transform::from_translation(grid_tf.to_world(coord)),
        ));
    }

    // enemies
    for e in level.enemies {
        let coord = GridCoord::new(e.x, e.y);
        commands.spawn((
            Actor,
            AI,
            Blocking,
            Position(coord),
            PendingIntent(Intent::Wait),
            Sprite {
                color: Color::srgb(0.8, 0.2, 0.8), 
                custom_size: Some(Vec2::splat(grid_tf.tile_size)),
                ..Default::default()
            },
            Transform::from_translation(grid_tf.to_world(coord)),
        ));
    }
}

pub fn sync_transforms(
    grid_transform: Res<GridTransform>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    for (pos, mut transform) in &mut q {
        transform.translation = grid_transform.to_world(pos.0);
    }
}


fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Text2d::new("Turn: 0"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(-380.0, 260.0, 10.0),
        TurnHudText,
    ));
}

fn update_turn_hud(
    turn: Res<TurnNumber>,
    mut q: Query<&mut Text2d, With<TurnHudText>>,
) {
    if !turn.is_changed() {
        return;
    }

    for mut text in &mut q {
        text.clear();
        text.push_str(&format!("Turn: {}", turn.0));
    }
}

fn teardown_game(
    mut commands: Commands,
    q_actors: Query<Entity, With<Actor>>,
    q_hud: Query<Entity, With<TurnHudText>>,
) {
    // Despawn all game actors 
    for e in &q_actors {
        commands.entity(e).despawn();
    }
    // Despawn HUD text
    for e in &q_hud {
        commands.entity(e).despawn();
    }
}
