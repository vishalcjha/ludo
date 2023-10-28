use serde::{Deserialize, Serialize};

use super::player::Player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    id: u32,
    status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Status {
    Created,
    ColorSelection { players: Vec<Player> },
    InProgress { players: Vec<Player> },
    Abandoned,
    Completed { players: Vec<Player> },
}
