use super::color::Color;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Command {
    CreateGame,
    AvailableColors,
    SelectColor(Color),
    StartGame { id: u32 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Response {
    AvailableColols { colors: Vec<Color> },
    CreateGameResponse { game_id: u32 },
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
}

#[cfg(test)]
mod test {
    use super::Command;

    #[test]
    fn test_serde() {
        let command = serde_json::to_string(&Command::AvailableColors).unwrap();

        let original_command = serde_json::from_str::<Command>(&command).unwrap();

        assert_eq!(original_command, Command::AvailableColors);
    }
}
