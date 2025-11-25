pub mod replay;
pub mod schedule;

use bevy::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use schedule::{TurnStep, TurnSystems};

use replay::ReplayLog;
use crate::grid::OccupancyIndex;
use crate::intents::{gather_player_input, plan_ai};

#[derive(Resource, Debug, Clone, Copy)]
pub struct TurnNumber(pub u64);

#[derive(Resource, Debug)]
pub struct TurnRng(pub StdRng);

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnNumber>()
            .insert_resource(TurnRng(StdRng::seed_from_u64(0)))
            .insert_resource(ReplayLog::default())        // input logging / replays
            .insert_resource(OccupancyIndex::default())   // grid occupancy queries
            // Configure deterministic turn pipeline inside FixedUpdate.
            .configure_sets(
                bevy::prelude::FixedUpdate,
                (
                    TurnSystems::Input,
                    TurnSystems::AiPlan,
                    TurnSystems::Resolve,
                    TurnSystems::Commit,
                    TurnSystems::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    // Stubs – real logic is added from other modules/plugins.
                    crate::intents::gather_player_input.in_set(TurnSystems::Input),
                    crate::intents::plan_ai.in_set(TurnSystems::AiPlan),
                    crate::engine::schedule::resolve_conflicts.in_set(TurnSystems::Resolve),
                    crate::engine::schedule::commit_changes.in_set(TurnSystems::Commit),
                    crate::engine::schedule::cleanup_turn.in_set(TurnSystems::Cleanup),
                    // crate::grid::rebuild_occupancy.in_set(TurnSystems::Cleanup),
                    bump_turn_number,
                )
                    .run_if(crate::scenes::is_in_game_scene),
            );
    }
}

fn bump_turn_number(mut turns: ResMut<TurnNumber>) {
    turns.0 += 1;
}

impl Default for TurnNumber {
    fn default() -> Self {
        Self(0)
    }
}
