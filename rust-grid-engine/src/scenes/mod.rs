use crate::components::*;
use crate::engine::TurnNumber;
use crate::engine::rules::{GetCaught, ReachedGoal};
use crate::grid::{GridCoord, GridTransform};
use crate::intents::Intent;
use crate::map::load_level_from_json;
use bevy::prelude::*;
use bevy::image::Image;
use bevy::asset::AssetServer;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use std::fs;
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
    LoadGame,
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
    SaveGame,
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

//LEVEL Loader
#[derive(Resource)]
pub struct LevelProgress {
    pub level_paths: Vec<String>,
    pub current: usize,
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            level_paths: vec![
                "assets/levels/level1.json".to_string(),
                "assets/levels/level2.json".to_string(),
                // add more here later
            ],
            current: 0,
        }
    }
}

// Pass level
#[derive(Debug, Clone, Copy)]
enum LevelCompleteItemKind {
    NextLevel,
    ExitGame,
}

#[derive(Component)]
struct LevelCompleteRoot;

#[derive(Component)]
struct LevelCompleteItem {
    index: usize,
    kind: LevelCompleteItemKind,
}

#[derive(Resource, Default)]
struct LevelCompleteSelection {
    index: usize,
}
// Save
#[derive(Resource, Default)]
pub struct SaveSlot {
    pub has_save: bool,
    pub level_index: usize,
}

// Game Over
#[derive(Component)]
struct GameOverRoot;

//images
#[derive(Resource)]
pub struct SpriteAssets {
    pub player: Handle<Image>,
    pub wall: Handle<Image>,
    pub goal: Handle<Image>,
    pub trap: Handle<Image>,
    pub door_locked: Handle<Image>,
    pub door_unlocked: Handle<Image>,
    pub enemy: Handle<Image>,
}

fn load_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpriteAssets {
        player:        asset_server.load("sprites/player.png"),
        wall:          asset_server.load("sprites/goal.png"),
        goal:          asset_server.load("sprites/goal.png"),
        trap:          asset_server.load("sprites/goal.png"),
        door_locked:   asset_server.load("sprites/goal.png"),
        door_unlocked: asset_server.load("sprites/goal.png"),
        enemy:         asset_server.load("sprites/goal.png"),
    });
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .insert_resource(GridTransform::default())
            .insert_resource(PauseState::default())
            .insert_resource(MainMenuSelection::default())
            .insert_resource(PauseMenuSelection::default())
            .insert_resource(LevelProgress::default())
            .insert_resource(LevelCompleteSelection::default())
            .insert_resource(SaveSlot::default())
            .add_systems(Startup, (setup_camera, load_sprites))
            // Menu enter/exit
            .add_systems(OnEnter(GameScene::Menu), setup_menu)
            .add_systems(OnExit(GameScene::Menu), teardown_menu)
            // InGame enter/exit
            .add_systems(OnEnter(GameScene::InGame), (setup_game, setup_hud))
            .add_systems(OnExit(GameScene::InGame), teardown_game)
            // GameOver
            .add_systems(OnEnter(GameScene::GameOver), setup_game_over)
            .add_systems(OnExit(GameScene::GameOver), teardown_game_over)
            .add_systems(
                Update,
                (
                    // MAIN MENU
                    (menu_input_system, update_menu_visuals).run_if(in_state(GameScene::Menu)),
                    // PAUSE MENU
                    (
                        pause_input_system,
                        pause_menu_navigation_system,
                        update_pause_menu_visuals,
                    )
                        .run_if(in_state(GameScene::InGame)),
                    // LEVEL COMPLETE MENU
                    level_complete_navigation_system,
                    update_level_complete_visuals,
                    
                    // Game over input (in GameOver scene)
                    game_over_input_system.run_if(in_state(GameScene::GameOver)),
                    // freeze when paused
                    (
                        sync_transforms,
                        update_turn_hud,
                        handle_goal_reached_events,
                        handle_get_caught,
                    )
                        .run_if(in_game_and_not_paused),
                ),
            );
    }
}

pub fn is_in_game_scene(state: Res<State<GameScene>>) -> bool {
    *state.get() == GameScene::InGame
}

pub fn in_game_and_not_paused(state: Res<State<GameScene>>, pause: Res<PauseState>) -> bool {
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

    spawn_item(
        &mut commands,
        0,
        MainMenuItemKind::NewGame,
        "New Game",
        30.0,
    );
    spawn_item(
        &mut commands,
        1,
        MainMenuItemKind::LoadGame,
        "Load Game",
        -10.0,
    );
    spawn_item(
        &mut commands,
        2,
        MainMenuItemKind::Settings,
        "Settings",
        -50.0,
    );
    spawn_item(&mut commands, 3, MainMenuItemKind::Exit, "Exit", -90.0);
}

fn menu_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MainMenuSelection>,
    q_items: Query<&MainMenuItem>,
    mut next: ResMut<NextState<GameScene>>,
    mut progress: ResMut<LevelProgress>,
    save_slot: Res<SaveSlot>,
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
                    progress.current = 0;
                    next.set(GameScene::InGame);
                }
                MainMenuItemKind::LoadGame => {
                    if save_slot.has_save {
                        progress.current = save_slot.level_index;
                        next.set(GameScene::InGame);
                    } else {
                        info!("No saved game yet.");
                    }
                }
                MainMenuItemKind::Settings => {
                    info!("Settings TBD");
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
    progress: Res<LevelProgress>,
    sprite_assets: Res<SpriteAssets>,
) {
    spawn_current_level(&mut commands, &grid_tf, &mut turn, &progress,&sprite_assets, );
}

fn spawn_current_level(
    commands: &mut Commands,
    grid_tf: &GridTransform,
    turn: &mut TurnNumber,
    progress: &LevelProgress,
    sprite_assets: &SpriteAssets,
) {
    // Reset per-run state
    turn.0 = 0;

    // Pick current level path
    let path = progress
        .level_paths
        .get(progress.current)
        .expect("LevelProgress.current out of range");

    let bytes = fs::read(path).unwrap_or_else(|e| {
        panic!("Failed to read level file {path}: {e}");
    });
    let level = load_level_from_json(&bytes).expect("invalid level JSON");

    // player
    let p = level.player_start;
    commands.spawn((
        Player,
        Actor,
        Position(p),
        PendingIntent(Intent::Wait),
        Sprite {
            image: sprite_assets.player.clone(),
            custom_size: Some(Vec2::splat(grid_tf.tile_size)),
            ..Default::default()
        },
        Transform::from_translation(grid_tf.to_world(p)),
    ));

    //walls
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
                image: sprite_assets.goal.clone(),
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

fn update_turn_hud(turn: Res<TurnNumber>, mut q: Query<&mut Text2d, With<TurnHudText>>) {
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
    q_level_complete_ui: Query<Entity, With<LevelCompleteRoot>>,
) {
    pause.paused = false;
    for e in &q_world {
        commands.entity(e).despawn();
    }
    for e in &q_pause_ui {
        commands.entity(e).despawn();
    }
    for e in &q_level_complete_ui {
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

            // Background
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
            spawn_pause_item(&mut commands, 0, PauseMenuItemKind::Resume, "Resume", 30.0);
            spawn_pause_item(
                &mut commands,
                1,
                PauseMenuItemKind::SaveGame,
                "Save Game",
                -10.0,
            );
            spawn_pause_item(
                &mut commands,
                2,
                PauseMenuItemKind::BackToMenu,
                "Back to Main Menu",
                -50.0,
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
    mut save_slot: ResMut<SaveSlot>,
    progress: Res<LevelProgress>,
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
                PauseMenuItemKind::SaveGame => {
                    // Save current level
                    save_slot.has_save = true;
                    save_slot.level_index = progress.current;
                    info!("Game saved at level index {}", progress.current);
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

fn handle_get_caught(
    mut caught_reader: MessageReader<GetCaught>,
    mut next: ResMut<NextState<GameScene>>,
) {
    for _event in caught_reader.read() {
        // On first caught event, go to GameOver
        next.set(GameScene::GameOver);
        break;
    }
}

fn handle_goal_reached_events(
    mut ev_goal: MessageReader<ReachedGoal>,
    mut pause: ResMut<PauseState>,
    mut selection: ResMut<LevelCompleteSelection>,
    mut commands: Commands,
    progress: Res<LevelProgress>,
    mut next: ResMut<NextState<GameScene>>,
    q_window: Query<Entity, With<LevelCompleteRoot>>,
) {
    // If a level-complete window is already visible, don't spawn another
    if !q_window.is_empty() {
        // still drain the events to avoid buildup
        for _ in ev_goal.read() {}
        return;
    }

    // We only care whether at least one goal event happened this frame
    let mut triggered = false;
    for _event in ev_goal.read() {
        triggered = true;
        break;
    }

    if !triggered {
        return;
    }

    let is_last_level = progress.current + 1 >= progress.level_paths.len();

    if is_last_level {
        // Last level -> go straight to GameOver
        next.set(GameScene::GameOver);
    } else {
        // More levels -> show "Level Complete" window
        pause.paused = true;
        selection.index = 0;
        spawn_level_complete_window(&mut commands);
    }
}

fn spawn_level_complete_window(commands: &mut Commands) {
    // background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(360.0, 220.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 70.0),
        LevelCompleteRoot,
    ));
    // text and button
    commands.spawn((
        Text2d::new("Level Complete!"),
        TextFont::from_font_size(32.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 70.0, 80.0),
        LevelCompleteRoot,
    ));
    spawn_level_complete_item(
        commands,
        0,
        LevelCompleteItemKind::NextLevel,
        "Next Level",
        20.0,
    );
    spawn_level_complete_item(
        commands,
        1,
        LevelCompleteItemKind::ExitGame,
        "Exit Game",
        -20.0,
    );
}

fn spawn_level_complete_item(
    commands: &mut Commands,
    index: usize,
    kind: LevelCompleteItemKind,
    label: &str,
    y: f32,
) {
    commands.spawn((
        Text2d::new(label),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, y, 80.0),
        LevelCompleteRoot,
        LevelCompleteItem { index, kind },
    ));
}

fn level_complete_navigation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<LevelCompleteSelection>,
    mut pause: ResMut<PauseState>,
    mut progress: ResMut<LevelProgress>,
    mut next: ResMut<NextState<GameScene>>,
    mut commands: Commands,
    q_items: Query<&LevelCompleteItem>,
    q_roots: Query<Entity, With<LevelCompleteRoot>>,
    q_world: Query<Entity, Or<(With<Position>, With<Actor>, With<TurnHudText>)>>,
    grid_tf: Res<GridTransform>,
    mut turn: ResMut<TurnNumber>,
    sprite_assets: Res<SpriteAssets>,
) {
    // Only run if the window is visible
    if q_roots.is_empty() {
        return;
    }

    // Find the maximum index to clamp selection
    let max_index = q_items.iter().map(|c| c.index).max().unwrap_or(0);

    // Navigation
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

    // Activate current selection
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        if let Some(item) = q_items.iter().find(|c| c.index == selection.index) {
            match item.kind {
                LevelCompleteItemKind::NextLevel => {
                    if progress.current + 1 < progress.level_paths.len() {
                        // Advance to the next level
                        progress.current += 1;

                        // Clear current game world + the level complete window
                        for e in &q_world {
                            commands.entity(e).despawn();
                        }
                        for e in &q_roots {
                            commands.entity(e).despawn();
                        }

                        // Unpause and spawn the next level directly
                        pause.paused = false;
                        spawn_current_level(
                            &mut commands,
                            &grid_tf,
                            &mut turn,
                            &progress,
                            &sprite_assets,
                        );
                    } else {
                        // No more levels: go to GameOver scene
                        pause.paused = false;
                        for e in &q_roots {
                            commands.entity(e).despawn();
                        }
                        next.set(GameScene::GameOver);
                    }
                }
                LevelCompleteItemKind::ExitGame => {
                    // Back to main menu
                    pause.paused = false;

                    // Clear world + window
                    for e in &q_world {
                        commands.entity(e).despawn();
                    }
                    for e in &q_roots {
                        commands.entity(e).despawn();
                    }

                    next.set(GameScene::Menu);
                }
            }
        }
    }
}


fn update_level_complete_visuals(
    selection: Res<LevelCompleteSelection>,
    mut q: Query<(&LevelCompleteItem, &mut TextColor)>,
) {
    for (item, mut color) in &mut q {
        color.0 = if item.index == selection.index {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::WHITE
        };
    }
}

fn setup_game_over(mut commands: Commands) {
    // Background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(360.0, 220.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 70.0),
        GameOverRoot,
    ));
    commands.spawn((
        Text2d::new("All Levels Complete!"),
        TextFont::from_font_size(32.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 60.0, 80.0),
        GameOverRoot,
    ));
    commands.spawn((
        Text2d::new("Press ENTER to return to menu"),
        TextFont::from_font_size(24.0),
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Transform::from_xyz(0.0, 10.0, 80.0),
        GameOverRoot,
    ));
}

fn teardown_game_over(mut commands: Commands, q: Query<Entity, With<GameOverRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn game_over_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameScene>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        next.set(GameScene::Menu);
    }
}
