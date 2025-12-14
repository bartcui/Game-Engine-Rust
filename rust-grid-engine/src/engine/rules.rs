use crate::engine::schedule::TurnSystems;
use crate::grid::occupancy::OccupancyIndex;
use crate::grid::{GridCoord, Layer};
use crate::intents::InputEvent;
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy)]
pub struct ReachedGoal(pub Entity);
#[derive(Message, Debug, Clone, Copy)]
pub struct SteppedOnTrap(pub Entity);
#[derive(Message, Debug, Clone, Copy)]
pub struct GetCaught(pub Entity);
/// Result of checking a move.
#[derive(Debug, Clone, Copy)]
pub enum MoveCheck {
    Allow,
    Blocked,
}

/// Rules that can swap out to change gameplay without touching engine code.
pub trait Rules: Send + Sync + 'static {
    fn can_enter(
        &self,
        occ: &OccupancyIndex,
        mover: Entity,
        from: GridCoord,
        to: GridCoord,
    ) -> MoveCheck;
}

/// Default rules: blocks `Blocking` or closed `Door`; fires `ReachedGoal`/`SteppedOnTrap`.
#[derive(Resource, Default)]
pub struct DefaultRules;

impl Rules for DefaultRules {
    fn can_enter(
        &self,
        occ: &OccupancyIndex,
        _mover: Entity,
        _from: GridCoord,
        to: GridCoord,
    ) -> MoveCheck {
        // Blockers block
        if !occ.at(Layer::Blockers, to).is_empty() || !occ.at(Layer::Actors, to).is_empty() {
            return MoveCheck::Blocked;
        }

        MoveCheck::Allow
    }
}

/// Resource holding the active rules object.
#[derive(Resource)]
pub struct ActiveRules(pub Box<dyn Rules>);

pub struct RulesPlugin;
impl Plugin for RulesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultRules>()
            .insert_resource(ActiveRules(Box::new(DefaultRules)))
            .add_message::<ReachedGoal>()
            .add_message::<SteppedOnTrap>()
            .add_message::<GetCaught>()
            .add_message::<InputEvent>()
            .add_systems(
                Update,
                (
                    crate::intents::plan_ai.in_set(TurnSystems::AiPlan),
                    crate::grid::rebuild_occupancy.in_set(TurnSystems::Resolve),
                    super::schedule::validate_moves.in_set(TurnSystems::Resolve),
                    super::schedule::commit_changes.in_set(TurnSystems::Commit),
                    super::schedule::fire_on_enter_hooks.in_set(TurnSystems::Commit),
                    super::schedule::cleanup_turn.in_set(TurnSystems::Cleanup),
                )
                    .chain()
                    .run_if(crate::scenes::in_game_and_not_paused)
                    .run_if(super::schedule::player_has_actions),
            );
    }
}
