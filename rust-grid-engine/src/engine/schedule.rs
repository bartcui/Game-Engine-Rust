use crate::components::Actor;
use crate::components::{Goal, PendingIntent, Position, Trap};
use crate::engine::rules::{ActiveRules, MoveCheck, ReachedGoal, SteppedOnTrap};
use crate::intents::Intent;
use crate::grid::occupancy::OccupancyIndex;
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

/// Validate PendingIntent::Move against Rules.
/// Converts illegal moves into Wait, legal ones kept as-is.
pub fn validate_moves(
    occ: Res<OccupancyIndex>,
    rules: Res<ActiveRules>,
    mut q: Query<(Entity, &Position, &mut PendingIntent), With<Actor>>,
) {
    let occ = &*occ;

    for (e, pos, mut pi) in q.iter_mut() {
        if let Intent::Move(dir) = pi.0 {
            let to = dir.step(pos.0);
            match rules.0.can_enter(occ, e, pos.0, to) {
                MoveCheck::Allow => { /* keep as is */ }
                MoveCheck::Blocked => {
                    pi.0 = Intent::Wait;
                }
            }
        }
    }
}

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

pub fn fire_on_enter_hooks(
    mut ev_goal: MessageWriter<ReachedGoal>,
    mut ev_trap: MessageWriter<SteppedOnTrap>,
    q_actor: Query<(Entity, &Position), With<Actor>>,
    q_goal: Query<&Position, With<Goal>>,
    q_trap: Query<&Position, With<Trap>>,
) {
    for (entity, pos) in q_actor.iter() {
        let at = pos.0;

        let has_goal = q_goal.iter().any(|g| g.0 == at);
        let has_trap = q_trap.iter().any(|t| t.0 == at);

        if has_goal {
            ev_goal.write(ReachedGoal(entity));
        }
        if has_trap {
            ev_trap.write(SteppedOnTrap(entity));
        }
    }
}

pub fn cleanup_turn(mut q: Query<&mut PendingIntent>) {
    for mut intent in &mut q {
        intent.0 = Intent::Wait;
    }
}
