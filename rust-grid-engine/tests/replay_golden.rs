use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use rust_grid_engine::engine::replay::{ActiveReplay, Replay, ReplayConfig, ReplayTickTimer};
use rust_grid_engine::engine::{EnginePlugin, TurnNumber};
use rust_grid_engine::scenes::{GameScene, PauseState, ScenePlugin};

// Adjust these paths to your actual components/types:
use rust_grid_engine::components::{AI, Actor, Blocking, Door, Goal, Player, Position, Trap};
use rust_grid_engine::engine::rules::GetCaught;
use rust_grid_engine::grid::GridCoord;
use rust_grid_engine::map::load_level_from_json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Snapshot {
    turn: u64,
    player: GridCoord,
    ais: Vec<GridCoord>,
    caught: bool,
}

// Resource that stores captured snapshots during a run
#[derive(Resource, Default)]
struct SnapshotLog(Vec<Snapshot>);

// Track last captured turn so we record once per committed turn
#[derive(Resource, Default)]
struct LastCapturedTurn(Option<u64>);

fn capture_snapshot_system(
    turn: Res<TurnNumber>,
    q_player: Query<&Position, With<Player>>,
    q_ai: Query<&Position, With<AI>>,
    mut caught_reader: MessageReader<GetCaught>,
    mut log: ResMut<SnapshotLog>,
    mut last: ResMut<LastCapturedTurn>,
) {
    let caught = caught_reader.read().next().is_some();

    let should_capture = last.0 != Some(turn.0) || caught;
    if !should_capture {
        return;
    }
    last.0 = Some(turn.0);

    let player = q_player.single().map(|p| p.0).unwrap_or(GridCoord::ZERO);

    let mut ais: Vec<GridCoord> = q_ai.iter().map(|p| p.0).collect();
    ais.sort_by_key(|c| (c.x, c.y));

    log.0.push(Snapshot {
        turn: turn.0,
        player,
        ais,
        caught,
    });
}

fn setup_test_world(world: &mut World) {
    use rust_grid_engine::components::{AI, PendingIntent, Player, Position};
    use rust_grid_engine::grid::GridCoord;
    use rust_grid_engine::intents::Intent;

    let path = "assets/levels/level1.json";
    let bytes = fs::read(path).unwrap_or_else(|e| {
        panic!("Failed to read level file {path}: {e}");
    });

    let level = load_level_from_json(&bytes).expect("invalid level JSON");

    // player
    let p = level.player_start;
    world.spawn((Player, Actor, Position(p), PendingIntent(Intent::Wait)));

    //walls
    for w in level.walls {
        world.spawn((Blocking, Position(w)));
    }

    // goals
    for g in level.goals {
        world.spawn((Goal, Position(g)));
    }

    // traps
    for t in level.traps {
        world.spawn((Trap, Position(t)));
    }

    // doors
    for d in level.doors {
        let coord = GridCoord::new(d.x, d.y);
        world.spawn((Door, Blocking, Position(coord)));
    }

    // enemies
    for e in level.enemies {
        let coord = GridCoord::new(e.x, e.y);
        world.spawn((
            Actor,
            AI,
            Blocking,
            Position(coord),
            PendingIntent(Intent::Wait),
        ));
    }
}

fn run_replay_and_capture(replay: &Replay) -> Vec<Snapshot> {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<rust_grid_engine::scenes::GameScene>();

    app.add_plugins(EnginePlugin);

    {
        let world = app.world_mut();
        setup_test_world(world);
    }

    // Deterministic “instant replay tick” for tests
    app.insert_resource(ReplayConfig { tick_seconds: 0.0 });
    app.insert_resource(PauseState { paused: false });

    // Snapshot resources + capture system (run late; Cleanup set is ideal if you have it)
    app.init_resource::<SnapshotLog>();
    app.init_resource::<LastCapturedTurn>();
    app.init_resource::<ButtonInput<KeyCode>>();

    // If you have TurnSystems::Cleanup, use that. Otherwise just add to Update.
    app.add_systems(Update, capture_snapshot_system);

    // Force the game into Replay state
    {
        app.world_mut()
            .resource_mut::<NextState<rust_grid_engine::scenes::GameScene>>()
            .set(rust_grid_engine::scenes::GameScene::Replay);
    }
    app.update(); // apply state changes / OnEnter hooks

    // Inject replay directly (bypasses filesystem so the test is stable)
    {
        let mut active = app.world_mut().resource_mut::<ActiveReplay>();
        active.start(replay.clone());
    }

    // Reset turn to first recorded turn (prevents “stuck at 0” if replay begins at 1+)
    {
        let first_turn = replay.inputs.first().map(|r| r.turn).unwrap_or(0);
        app.world_mut().resource_mut::<TurnNumber>().0 = first_turn;
    }

    // Reset timer for cleanliness (even though tick_seconds=0 bypasses it)
    {
        app.world_mut().resource_mut::<ReplayTickTimer>().0.reset();
    }

    // Run until replay exhausted or max steps reached
    let max_steps = 50_000;
    for _ in 0..max_steps {
        app.update();

        let done = {
            let active = app.world().resource::<ActiveReplay>();
            active
                .replay
                .as_ref()
                .map(|r| active.cursor >= r.inputs.len())
                .unwrap_or(true)
        };

        if done {
            break;
        }
    }

    // Return snapshots
    app.world_mut().remove_resource::<SnapshotLog>().unwrap().0
}

#[test]
fn replay_is_deterministic_twice() {
    // Load replay (seed + inputs)
    let bytes =
        fs::read("assets/replays/last_run.json").expect("missing assets/replays/last_run.json");
    let replay: Replay = serde_json::from_slice(&bytes).expect("invalid replay json");

    // Run twice
    let a = run_replay_and_capture(&replay);
    let b = run_replay_and_capture(&replay);

    assert_eq!(a, b);
}

#[test]
fn replay_matches_golden() {
    let bytes =
        fs::read("assets/replays/last_run.json").expect("missing assets/replays/last_run.json");
    let replay: Replay = serde_json::from_slice(&bytes).expect("invalid replay json");

    let actual = run_replay_and_capture(&replay);

    let golden_bytes = fs::read("assets/replays/golden_snapshots.json")
        .expect("missing assets/replays/golden_snapshots.json (generate it once)");
    let expected: Vec<Snapshot> =
        serde_json::from_slice(&golden_bytes).expect("invalid golden snapshot json");

    assert_eq!(actual, expected);
}

// Helper you run locally once to generate/update the golden file.
// Keep it ignored so CI doesn't overwrite.
#[test]
#[ignore]
fn generate_golden_snapshots() {
    let bytes =
        fs::read("assets/replays/last_run.json").expect("missing assets/replays/last_run.json");
    let replay: Replay = serde_json::from_slice(&bytes).expect("invalid replay json");

    let snaps = run_replay_and_capture(&replay);

    fs::write(
        "assets/replays/golden_snapshots.json",
        serde_json::to_vec_pretty(&snaps).unwrap(),
    )
    .unwrap();
}
