pub mod replay;
pub mod rules;
pub mod schedule;

use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng};
use schedule::TurnSystems;

use crate::grid::OccupancyIndex;
use replay::ReplayLog;

#[derive(Resource, Debug, Clone, Copy)]
pub struct TurnNumber(pub u64);

#[derive(Resource, Debug)]
pub struct TurnRng(pub StdRng);

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnNumber>()
            .insert_resource(TurnRng(StdRng::seed_from_u64(0)))
            .insert_resource(ReplayLog::default()) // input logging / replays
            .insert_resource(OccupancyIndex::default()) // grid occupancy queries
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
                Update,
                crate::intents::gather_player_input
                    .in_set(TurnSystems::Input)
                    .run_if(crate::scenes::in_game_and_not_paused),
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
