use crate::grid::GridCoord;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position(pub GridCoord);

#[derive(Component, Debug, Clone, Copy)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub struct AI;

#[derive(Component, Debug, Clone, Copy)]
pub struct Blocking;

#[derive(Component, Debug, Clone, Copy)]
pub struct Goal;

#[derive(Component, Debug, Clone, Copy)]
pub struct Trap;

#[derive(Component, Debug, Clone, Copy)]
pub struct Actor; 

#[derive(Component)]
pub struct Door;

// Temporary per-turn intent buffer
#[derive(Component, Debug, Clone)]
pub struct PendingIntent(pub crate::intents::Intent);
