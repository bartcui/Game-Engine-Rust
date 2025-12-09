use crate::components::{PendingIntent, Player};
use crate::grid::{Dir};
use bevy::{input::keyboard::KeyCode, prelude::*};
use serde::{Deserialize, Serialize};
use crate::engine::replay::{RecordedInput, ReplayLog};
use crate::engine::TurnNumber;

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
    turn: ResMut<TurnNumber>
) {
    // updated
    let input_event = if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
    {
        Some(InputEvent::Move(Dir::Up))
    } else if keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
    {
        Some(InputEvent::Move(Dir::Down))
    } else if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
    {
        Some(InputEvent::Move(Dir::Left))
    } else if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
    {
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

pub fn plan_ai(mut q_ai: Query<&mut PendingIntent, With<crate::components::AI>>) {
    for mut intent in &mut q_ai {
        // placeholder AI: do nothing
        intent.0 = Intent::Wait;
    }
}
