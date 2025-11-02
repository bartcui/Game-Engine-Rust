use crate::components::{PendingIntent, Player};
use crate::grid::{Dir, GridCoord};
use bevy::{input::keyboard::KeyCode, prelude::*};
use serde::{Deserialize, Serialize};

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
    mut q_players: Query<(Entity, &mut PendingIntent), With<Player>>,
    mut commands: Commands,
) {
    // Simple mapping – expand later
    let dir = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        Some(Dir::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(Dir::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(Dir::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(Dir::Right)
    } else {
        None
    };

    if let Some(d) = dir {
        for (e, mut pending) in q_players.iter_mut() {
            pending.0 = Intent::Move(d);
            commands.entity(e).insert(PendingIntent(Intent::Move(d)));
        }
    }
}

pub fn plan_ai(mut q_ai: Query<&mut PendingIntent, With<crate::components::AI>>) {
    for mut intent in &mut q_ai {
        // placeholder AI: do nothing
        intent.0 = Intent::Wait;
    }
}
