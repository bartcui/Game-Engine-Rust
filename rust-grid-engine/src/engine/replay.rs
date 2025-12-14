use super::TurnNumber;
use crate::components::{PendingIntent, Player};
use crate::engine::{RunSeed, TurnRng};
use crate::intents::{InputEvent, Intent};
use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Resource, Debug, Clone, Copy)]
pub struct ReplayConfig {
    /// Set to 0.0 in tests to inject every update deterministically.
    pub tick_seconds: f32,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self { tick_seconds: 0.25 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Replay {
    pub seed: u64,
    pub inputs: Vec<RecordedInput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordedInput {
    pub turn: u64,
    pub input: super::super::intents::InputEvent,
}

#[derive(Resource, Default, Debug)]
pub struct ReplayLog(pub Vec<RecordedInput>);

#[derive(Resource)]
pub struct ReplayTickTimer(pub Timer);

impl Default for ReplayTickTimer {
    fn default() -> Self {
        // Default to one move every 0.25s (4 moves/sec).
        Self(Timer::from_seconds(0.25, TimerMode::Repeating))
    }
}

impl ReplayLog {
    pub fn record(&mut self, turn: u64, input: super::super::intents::InputEvent) {
        self.0.push(RecordedInput { turn, input });
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// Playback state: holds a loaded replay + cursor into its inputs.
#[derive(Resource, Default, Debug)]
pub struct ActiveReplay {
    pub replay: Option<Replay>,
    pub cursor: usize,
}

impl ActiveReplay {
    /// Start playing a replay from the beginning.
    pub fn start(&mut self, replay: Replay) {
        self.replay = Some(replay);
        self.cursor = 0;
    }

    /// Stop replay playback.
    pub fn stop(&mut self) {
        self.replay = None;
        self.cursor = 0;
    }

    pub fn is_active(&self) -> bool {
        self.replay.is_some()
    }
}

impl Replay {
    pub fn from_log(seed: u64, log: &ReplayLog) -> Self {
        Replay {
            seed,
            inputs: log.0.clone(),
        }
    }

    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let bytes = fs::read(path)?;
        let replay = serde_json::from_slice(&bytes)?;
        Ok(replay)
    }
}

pub fn is_replay_active(active: Res<ActiveReplay>) -> bool {
    active.is_active()
}

pub fn stop_replay_mode(mut active: ResMut<ActiveReplay>, mut tick_timer: ResMut<ReplayTickTimer>) {
    active.stop();
    tick_timer.0.reset();
}

pub fn reset_replay(
    mut log: ResMut<ReplayLog>,
    mut turn: ResMut<TurnNumber>,
    mut rng: ResMut<TurnRng>,
    mut seed: ResMut<RunSeed>,
) {
    // Fresh run => fresh turn counter + fresh log

    let new_seed: u64 = rand::rngs::OsRng.next_u64();

    seed.0 = new_seed;
    rng.0 = StdRng::seed_from_u64(new_seed);

    turn.0 = 0;
    log.clear();
}

/// System: feed recorded `InputEvent`s into the game for the current turn.
pub fn feed_replay_inputs_system(
    mut active: ResMut<ActiveReplay>,
    mut q_player: Query<&mut PendingIntent, With<Player>>,
    turn: Res<TurnNumber>,
    time: Res<Time>,
    cfg: Res<ReplayConfig>,
    mut tick_timer: ResMut<ReplayTickTimer>,
) {
    let current_turn = turn.0;

    let Some(replay) = active.replay.as_ref() else {
        return;
    };
    // Tick the timer; only inject input when a tick completes.
    if cfg.tick_seconds > 0.0 {
        tick_timer.0.tick(time.delta());
        if !tick_timer.0.just_finished() {
            return;
        }
    }

    let Ok(mut pending) = q_player.single_mut() else {
        return;
    };

    // Only apply if player is idle (prevents multiple actions per turn)
    if !matches!(pending.0, Intent::Wait) {
        return;
    }

    let mut cursor = active.cursor;
    let inputs = &replay.inputs;

    // Dispatch all inputs whose `turn` matches the current `TurnNumber`.
    while cursor < inputs.len() && inputs[cursor].turn == current_turn {
        let rec = &inputs[cursor];
        match &rec.input {
            InputEvent::Move(dir) => pending.0 = Intent::Move(*dir),
            InputEvent::Interact => pending.0 = Intent::Interact,
            _ => {}
        }
        cursor += 1;
    }
    active.cursor = cursor;
}

pub fn save_replay_on_game_over(mut log: ResMut<ReplayLog>, seed: Res<RunSeed>) {
    let replay = Replay::from_log(seed.0, &log);
    let _ = replay.save_to_file("assets/replays/last_run.json").ok();

    log.clear();
}

pub fn start_replay_mode(
    mut active: ResMut<ActiveReplay>,
    mut turn: ResMut<TurnNumber>,
    mut tick_timer: ResMut<ReplayTickTimer>,
    mut rng: ResMut<TurnRng>,
    mut seed: ResMut<RunSeed>,
) {
    // Load the replay file
    if let Ok(replay) = Replay::load_from_file("assets/replays/last_run.json") {
        // Reset the speed timer
        tick_timer.0.reset();

        // Restore seed
        seed.0 = replay.seed;
        rng.0 = StdRng::seed_from_u64(replay.seed);

        turn.0 = 0;
        active.start(replay);
    } else {
        active.stop();
    }
}
