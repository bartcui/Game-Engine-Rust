use crate::components::{PendingIntent, Position};
use crate::intents::Intent;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnSystems {
    /// Convert input to intents
    Input,
    /// AI planning to intents
    AiPlan,
    /// Tie-breakers, collision rules
    Resolve,
    /// The **only** place that mutates canonical state
    Commit,
    /// Cleanup temp state, events, buffers
    Cleanup,
}

/// Label handy for tests/tools that want to insert at precise moments
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum TurnStep {
    #[default]
    Idle,
    Input,
    AiPlan,
    Resolve,
    Commit,
    Cleanup,
}
//todo: implement resolve_conflicts
pub fn resolve_conflicts() {}

pub fn commit_changes(mut q: Query<(&mut Position, Option<&mut PendingIntent>)>) {
    for (mut pos, pending_intent) in &mut q {
        if let Some(mut intent) = pending_intent {
            match intent.0 {
                Intent::Move(dir) => {
                    pos.0 = dir.step(pos.0);
                }
                Intent::Wait => {
                    // do nothing
                }
                Intent::Interact => {
                    // placeholder
                }
            }
            // Clear intent after committing
            intent.0 = Intent::Wait;
        }
    }
}
pub fn cleanup_turn(mut q: Query<&mut PendingIntent>) {
    for mut intent in &mut q {
        intent.0 = Intent::Wait;
    }
}
