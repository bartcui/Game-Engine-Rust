use crate::components::{AI, PendingIntent, Player, Position};
use crate::engine::TurnNumber;
use crate::engine::replay::{RecordedInput, ReplayLog};
use crate::grid::GridCoord;
use crate::grid::occupancy::OccupancyIndex;
use crate::grid::{Dir, Layer};
use crate::pathfinding::astar::{AStarPolicy, astar};
use bevy::{input::keyboard::KeyCode, prelude::*};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    Move(Dir),
    Wait,
    Interact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    Move(Dir),
    Wait,
    Interact,
}

pub fn gather_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q_players: Query<&mut PendingIntent, With<Player>>,
    mut replay: ResMut<ReplayLog>,
    turn: ResMut<TurnNumber>,
) {
    // updated
    let input_event = if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
    {
        Some(InputEvent::Move(Dir::Up))
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(InputEvent::Move(Dir::Down))
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(InputEvent::Move(Dir::Left))
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(InputEvent::Move(Dir::Right))
    } else {
        None
    };

    if let Some(event) = input_event {
        // apply intent to the player
        for mut pending in &mut q_players {
            pending.0 = match event {
                InputEvent::Move(dir) => Intent::Move(dir),
                InputEvent::Wait => Intent::Wait,
                InputEvent::Interact => Intent::Interact,
            };
        }

        // record input with current turn number
        let current_turn = turn.0;
        replay.0.push(RecordedInput {
            turn: current_turn,
            input: event,
        });
    }
}

pub fn plan_ai(
    occ: Res<OccupancyIndex>,
    q_player: Query<&Position, With<Player>>,
    mut q_ai: Query<(&Position, &mut PendingIntent), With<AI>>,
) {
    let Ok(player_pos) = q_player.single() else {
        // no player -> AI does nothing
        return;
    };
    let target = player_pos.0;

    // Build a policy from occupancy.
    // clone minimal data into the closure to satisfy 'static.
    let occ_clone = occ.clone();

    let policy = AStarPolicy {
        passable: Arc::new(move |coord: GridCoord| {
            // Allow the goal tile itself so we can actually reach the player.
            if coord == target {
                return true;
            }
            occ_clone.at(Layer::Blockers, coord).is_empty()
                && occ_clone.at(Layer::Actors, coord).is_empty()
        }),
        cost: Arc::new(|_from: GridCoord, _to: GridCoord| 1),
    };

    for (pos, mut pending) in q_ai.iter_mut() {
        let start = pos.0;

        if start == target {
            // Already on player
            pending.0 = Intent::Wait;
            continue;
        }

        // Compute full path from AI to player
        let Some(path) = astar(start, target, &policy) else {
            // No path found → wait
            pending.0 = Intent::Wait;
            continue;
        };

        // Path is [start, step1, step2, ..., goal]
        if path.len() < 2 {
            // nothing to do
            pending.0 = Intent::Wait;
            continue;
        }

        let next = path[1];
        let dir = grid_step_to_dir(start, next);

        if let Some(dir) = dir {
            pending.0 = Intent::Move(dir);
        } else {
            // Next tile is weird → wait
            pending.0 = Intent::Wait;
        }
    }
}

// Helper: convert (start -> next) into Dir
fn grid_step_to_dir(from: GridCoord, to: GridCoord) -> Option<Dir> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    match (dx, dy) {
        (1, 0) => Some(Dir::Right),
        (-1, 0) => Some(Dir::Left),
        (0, 1) => Some(Dir::Up),
        (0, -1) => Some(Dir::Down),
        _ => None,
    }
}
