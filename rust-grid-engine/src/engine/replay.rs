use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

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

impl ReplayLog {
    pub fn record(&mut self, turn: u64, input: super::super::intents::InputEvent) {
        self.0.push(RecordedInput { turn, input });
    }
}

impl Replay {
    pub fn from_log(seed: u64, log: &ReplayLog) -> Self {
        Replay {
            seed,
            inputs: log.0.clone(),
        }
    }

    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let bytes = fs::read(path)?;
        let replay = serde_json::from_slice(&bytes)?;
        Ok(replay)
    }
}