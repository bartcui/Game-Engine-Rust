use crate::components::{AI, Actor, Player};
use crate::components::{Goal, PendingIntent, Position, Trap};
use crate::engine::TurnNumber;
use crate::engine::rules::{ActiveRules, GetCaught, MoveCheck, ReachedGoal, SteppedOnTrap};
use crate::grid::occupancy::OccupancyIndex;
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

pub fn commit_changes(
    mut q: Query<(&mut Position, Option<&mut PendingIntent>, Option<&Player>)>,
    mut turn: ResMut<TurnNumber>,
) {
    let mut player_moved_this_tick = false;

    for (mut pos, pending_intent, maybe_player) in &mut q {
        if let Some(mut intent) = pending_intent {
            match intent.0 {
                Intent::Move(dir) => {
                    pos.0 = dir.step(pos.0);

                    // only increment turn if player moved
                    if maybe_player.is_some() {
                        player_moved_this_tick = true;
                    }
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
    if player_moved_this_tick {
        turn.0 += 1;
    }
}

pub fn fire_on_enter_hooks(
    mut ev_goal: MessageWriter<ReachedGoal>,
    mut ev_trap: MessageWriter<SteppedOnTrap>,
    mut ev_caught: MessageWriter<GetCaught>,
    q_goal: Query<&Position, With<Goal>>,
    q_trap: Query<&Position, With<Trap>>,
    q_player: Query<(Entity, &Position), With<Player>>,
    q_ai: Query<&Position, With<AI>>,
) {
    if let Ok((player_ent, player_pos)) = q_player.single() {
        let player_at = player_pos.0;

        let has_goal = q_goal.iter().any(|g| {
            let dx = (g.0.x - player_at.x).abs();
            let dy = (g.0.y - player_at.y).abs();
            dx + dy == 0
        });
        let has_trap = q_trap.iter().any(|t| {
            let dx = (t.0.x - player_at.x).abs();
            let dy = (t.0.y - player_at.y).abs();
            dx + dy == 0
        });

        if has_goal {
            ev_goal.write(ReachedGoal(player_ent));
        }
        if has_trap {
            ev_trap.write(SteppedOnTrap(player_ent));
        }

        for ai_pos in q_ai.iter() {
            let dx = (ai_pos.0.x - player_at.x).abs();
            let dy = (ai_pos.0.y - player_at.y).abs();
            if dx + dy == 0 {
                ev_caught.write(GetCaught(player_ent));
                break;
            }
        }
    }
}

pub fn cleanup_turn(mut q: Query<&mut PendingIntent>) {
    for mut intent in &mut q {
        intent.0 = Intent::Wait;
    }
}

pub fn player_has_actions(q_player: Query<&PendingIntent, With<Player>>) -> bool {
    let Ok(pending) = q_player.single() else {
        return false;
    };
    match pending.0 {
        Intent::Move(_) | Intent::Interact => true,
        Intent::Wait => false,
    }
}
