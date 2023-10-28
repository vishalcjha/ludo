use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::server::entity::game::Game;

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
}
