use serde::{Deserialize, Serialize};

use super::color::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    color: Color,
    id: u8,
    status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Status {
    Home,
    Running { x: u8, y: u8 },
    Done,
}
