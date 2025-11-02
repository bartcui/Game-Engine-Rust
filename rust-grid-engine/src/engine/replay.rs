use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Replay {
    pub seed: u64,
    pub inputs: Vec<RecordedInput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordedInput {
    pub turn: u64,
    pub input: super::super::intents::InputEvent,
}

#[derive(Resource, Default, Debug)]
pub struct ReplayLog(pub Vec<RecordedInput>);
