use super::color::Color;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Command {
    CreateGame,
    AvailableColors { id: u32 },
    JoinGame { id: u32, color: Color },
    StartGame { id: u32 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Response {
    AvailableColols { colors: Vec<Color> },
    // Select Color can fail, but to limit ping pong of message, if possible we will return alternative color.
    PickedColor { color: Color },
    CreateGameResponse { game_id: u32 },
    FailureMessage { message: String },
}

impl Response {
    pub fn make_available_colors(colors: Vec<Color>) -> Response {
        Response::AvailableColols { colors }
    }

    pub fn all_color_response() -> Response {
        Response::AvailableColols {
            colors: vec![Color::Yellow, Color::Blue, Color::Red, Color::Green],
        }
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::Command;

    #[test]
    fn test_serde() {
        let game_id = 42;
        let command = serde_json::to_string(&Command::AvailableColors { id: game_id }).unwrap();

        let original_command = serde_json::from_str::<Command>(&command).unwrap();

        assert_eq!(original_command, Command::AvailableColors { id: game_id });
    }
}
