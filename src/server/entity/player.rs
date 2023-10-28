use serde::{Deserialize, Serialize};

use super::token::Token;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Player {
    tokens: [Token; 4],
    name: String,
}
