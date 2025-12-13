pub mod replay;
pub mod rules;
pub mod schedule;

use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng};
use schedule::TurnSystems;

use crate::engine::replay::{
    ActiveReplay, ReplayLog, ReplayTickTimer, feed_replay_inputs_system, is_replay_active,
};
use crate::engine::rules::GetCaught;
use crate::engine::rules::ReachedGoal as GoalReached;
use crate::grid::OccupancyIndex;
use crate::intents::InputEvent;

#[derive(Resource, Debug, Clone, Copy)]
pub struct TurnNumber(pub u64);

#[derive(Resource, Debug)]
pub struct TurnRng(pub StdRng);

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnNumber>()
            .init_resource::<ActiveReplay>()
            .init_resource::<ReplayTickTimer>()
            .insert_resource(TurnRng(StdRng::seed_from_u64(0)))
            .insert_resource(ReplayLog::default()) // input logging / replays
            .insert_resource(OccupancyIndex::default()) // grid occupancy queries
            // Configure deterministic turn pipeline inside Update.
            .configure_sets(
                Update,
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
                Update,
                (
                    crate::intents::gather_player_input
                        .in_set(TurnSystems::Input)
                        .run_if(crate::scenes::in_game_and_not_paused)
                        .run_if(not(is_replay_active)),
                    feed_replay_inputs_system
                        .in_set(TurnSystems::Input)
                        .run_if(crate::scenes::in_game_and_not_paused)
                        .run_if(is_replay_active),
                ),
            )
            // Plug the default rules & resolve/commit systems.
            .add_plugins(crate::engine::rules::RulesPlugin);
    }
}

impl Default for TurnNumber {
    fn default() -> Self {
        Self(0)
    }
}
