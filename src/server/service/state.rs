use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::server::entity::game::Game;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct AppState {
    games: Arc<Mutex<HashMap<u32, Game>>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_game(&self) -> Result<u32> {
        let mut current_game = self
            .games
            .lock()
            .map_err(|err| anyhow!(format!("Failed to lock with error {:#?}", err)))?;

        let next_key = current_game
            .keys()
            .max()
            .or(Some(&0))
            .ok_or_else(|| anyhow!("Failed to get max key"))?
            + 1;

        current_game.insert(next_key, Game::new(next_key));

        Ok(next_key)
    }
}
