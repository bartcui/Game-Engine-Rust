use super::types::Level;
use anyhow::Result;

pub fn load_level_from_json(bytes: &[u8]) -> Result<Level> {
    let lvl: Level = serde_json::from_slice(bytes)?;
    Ok(lvl)
}
