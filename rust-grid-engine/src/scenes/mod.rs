use crate::components::*;
use crate::engine::TurnNumber;
use crate::grid::{GridCoord,GridTransform};
use crate::intents::Intent;
use crate::map::load_level_from_json;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextFont, TextColor};
use std::fs;
use bevy::app::AppExit;
use std::process;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    InGame,
    GameOver,
}

#[derive(Component)]
pub struct TurnHudText;

// MAIN MENU

#[derive(Component)]
struct MenuText;         
#[derive(Debug, Clone, Copy)]
enum MainMenuItemKind {
    NewGame,
    Settings,
    Exit,
}

#[derive(Component)]
struct MainMenuItem {
    index: usize,
    kind: MainMenuItemKind,
}

#[derive(Resource, Default)]
struct MainMenuSelection {
    index: usize,
}

// PAUSE MENU

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

#[derive(Debug, Clone, Copy)]
enum PauseMenuItemKind {
    Resume,
    BackToMenu,
}

#[derive(Component)]
struct PauseMenuRoot; 

#[derive(Component)]
struct PauseMenuItem {
    index: usize,
    kind: PauseMenuItemKind,
}

#[derive(Resource, Default)]
struct PauseMenuSelection {
    index: usize,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .insert_resource(GridTransform::default())
            .insert_resource(PauseState::default())
            .insert_resource(MainMenuSelection::default())
            .insert_resource(PauseMenuSelection::default())
            .add_systems(Startup, setup_camera)
            // Menu enter/exit
            .add_systems(OnEnter(GameScene::Menu), setup_menu)
            .add_systems(OnExit(GameScene::Menu), teardown_menu)
            // InGame enter/exit
            .add_systems(OnEnter(GameScene::InGame), (setup_game, setup_hud))
            .add_systems(OnExit(GameScene::InGame), teardown_game)
            .add_systems(
                Update,
                (
                    // menu page
                    (menu_input_system, update_menu_visuals)
                        .run_if(in_state(GameScene::Menu)),
                    // pause window
                    (pause_input_system, pause_menu_navigation_system, update_pause_menu_visuals)
                        .run_if(in_state(GameScene::InGame)),
                    // system freeze when paused
                    (
                        sync_transforms,
                        crate::grid::rebuild_occupancy,
                        update_turn_hud,
                    )
                        .run_if(in_game_and_not_paused),
                ),
            );
    }
}

pub fn is_in_game_scene(state: Res<State<GameScene>>) -> bool {
    *state.get() == GameScene::InGame
}

pub fn in_game_and_not_paused(
    state: Res<State<GameScene>>,
    pause: Res<PauseState>,
) -> bool {
    *state.get() == GameScene::InGame && !pause.paused
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
// menu functions
fn setup_menu(mut commands: Commands, mut selection: ResMut<MainMenuSelection>) {
    selection.index = 0;

    // Title
    commands.spawn((
        Text2d::new("Demo"),
        TextFont::from_font_size(40.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 80.0, 10.0),
        MenuText,
    ));

    // helper function to spawn one menu item
    fn spawn_item(
        commands: &mut Commands,
        index: usize,
        kind: MainMenuItemKind,
        label: &str,
        y: f32,
    ) {
        commands.spawn((
            Text2d::new(label),
            TextFont::from_font_size(28.0),
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, y, 10.0),
            MenuText,
            MainMenuItem { index, kind },
        ));
    }

    spawn_item(&mut commands, 0, MainMenuItemKind::NewGame, "New Game", 20.0);
    spawn_item(&mut commands, 1, MainMenuItemKind::Settings, "Settings", -20.0);
    spawn_item(&mut commands, 2, MainMenuItemKind::Exit, "Exit", -60.0);
}

fn menu_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MainMenuSelection>,
    q_items: Query<&MainMenuItem>,
    mut next: ResMut<NextState<GameScene>>,
) {
    let max_index = q_items.iter().map(|c| c.index).max().unwrap_or(0);

    // direction
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        if selection.index > 0 {
            selection.index -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        if selection.index < max_index {
            selection.index += 1;
        }
    }

    // select
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        if let Some(item) = q_items.iter().find(|c| c.index == selection.index) {
            match item.kind {
                MainMenuItemKind::NewGame => {
                    next.set(GameScene::InGame);
                }
                MainMenuItemKind::Settings => {
                    info!("Settings to be implemented");
                }
                MainMenuItemKind::Exit => {
                    process::exit(0);
                }
            }
        }
    }
}

fn update_menu_visuals(
    selection: Res<MainMenuSelection>,
    mut q: Query<(&MainMenuItem, &mut TextColor)>,
) {
    for (item, mut color) in &mut q {
        color.0 = if item.index == selection.index {
            Color::srgb(1.0, 1.0, 0.0) 
        } else {
            Color::WHITE
        };
    }
}

fn teardown_menu(mut commands: Commands, q: Query<Entity, With<MenuText>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}


fn setup_game(
    mut commands: Commands,
    grid_tf: Res<GridTransform>,
    mut turn: ResMut<TurnNumber>, 
) {
    turn.0 = 0;
    // game file loaded here
    let bytes = fs::read("assets/levels/level2.json")
        .expect("failed to read assets/levels/level2.json");
    let level = load_level_from_json(&bytes).expect("invalid level JSON");

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
    mut pause: ResMut<PauseState>,
    q_world: Query<Entity, Or<(With<Position>, With<Actor>, With<TurnHudText>)>>,
    q_pause_ui: Query<Entity, With<PauseMenuRoot>>,
) {
    pause.paused = false;
    // despawn all game-world entities
    for e in &q_world {
        commands.entity(e).despawn();
    }
    for e in &q_pause_ui {
        commands.entity(e).despawn();
    }
}

fn pause_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    mut selection: ResMut<PauseMenuSelection>,
    mut commands: Commands,
    q_pause_root: Query<Entity, With<PauseMenuRoot>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        pause.paused = !pause.paused;

        if pause.paused {
            selection.index = 0;

            // Background panel
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(320.0, 200.0)),
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, 50.0),
                PauseMenuRoot,
            ));

            // Title
            commands.spawn((
                Text2d::new("Paused"),
                TextFont::from_font_size(32.0),
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 60.0, 60.0),
                PauseMenuRoot,
            ));

            // Buttons
            spawn_pause_item(&mut commands, 0, PauseMenuItemKind::Resume, "Resume", 20.0);
            spawn_pause_item(
                &mut commands,
                1,
                PauseMenuItemKind::BackToMenu,
                "Return to Main Menu",
                -20.0,
            );
        } else {
            // remove all pause menu UI
            for e in &q_pause_root {
                commands.entity(e).despawn();
            }
        }
    }
}

fn spawn_pause_item(
    commands: &mut Commands,
    index: usize,
    kind: PauseMenuItemKind,
    label: &str,
    y: f32,
) {
    commands.spawn((
        Text2d::new(label),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, y, 60.0),
        PauseMenuRoot,
        PauseMenuItem { index, kind },
    ));
}

fn pause_menu_navigation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    mut selection: ResMut<PauseMenuSelection>,
    mut next: ResMut<NextState<GameScene>>,
    q_items: Query<&PauseMenuItem>,
    q_roots: Query<Entity, With<PauseMenuRoot>>,
    mut commands: Commands,
) {
    if !pause.paused {
        return;
    }

    let max_index = q_items.iter().map(|c| c.index).max().unwrap_or(0);

    // direction
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        if selection.index > 0 {
            selection.index -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        if selection.index < max_index {
            selection.index += 1;
        }
    }

    // select
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        if let Some(item) = q_items.iter().find(|c| c.index == selection.index) {
            match item.kind {
                PauseMenuItemKind::Resume => {
                    pause.paused = false;
                }
                PauseMenuItemKind::BackToMenu => {
                    pause.paused = false;
                    next.set(GameScene::Menu);
                }
            }

            // Remove pause menu UI
            for e in &q_roots {
                commands.entity(e).despawn();
            }
        }
    }
}

fn update_pause_menu_visuals(
    pause: Res<PauseState>,
    selection: Res<PauseMenuSelection>,
    mut q: Query<(&PauseMenuItem, &mut TextColor)>,
) {
    if !pause.paused {
        return;
    }

    for (item, mut color) in &mut q {
        color.0 = if item.index == selection.index {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::WHITE
        };
    }
}

