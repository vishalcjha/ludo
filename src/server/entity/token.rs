use serde::{Deserialize, Serialize};

use super::color::Color;
use anyhow::{bail, Result};

/// There are 4 token of each color. Their ids are b/w 1 - 4.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token {
    color: Color,
    id: u8,
    status: Status,
}

/// Status of token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Status {
    /// Token can not be used yet, and they can only be used if dice says 6 and player chooses to bring them out.
    Home,
    /// Home token transition to Running after getting 6 points. They get 50 position to move.
    Running {
        pos: u8,
    },
    /// After 50 moves, they have 6 moves left. But now they are out of any danger.
    FinalWalk {
        pos: u8,
    },
    Done,
}

impl Token {
    pub fn new(color: Color, id: u8) -> Self {
        assert!(id >= 1 && id <= 4);
        Token {
            color,
            id,
            status: Status::Home,
        }
    }

    fn with_staus(self, status: Status) -> Self {
        Token {
            color: self.color,
            id: self.id,
            status,
        }
    }

    // fn with_pos(self, pos: u8) -> Self {
    //     let new_staus = match self.status {
    //         Status::Home => panic!("Home can not have pos"),
    //         Status::Running { .. } => Status::Running { pos },
    //         Status::FinalWalk { .. } => Status::FinalWalk { pos },
    //         Status::Done => panic!("Done can not have pos"),
    //     };
    //     Token {
    //         color: self.color,
    //         id: self.id,
    //         status: new_staus,
    //     }
    // }

    pub fn is_valid_move(&self, count: u8) -> bool {
        match self.status {
            Status::Home => count == 6,
            Status::Running { pos } => pos + count <= 57,
            Status::FinalWalk { pos } => pos + count <= 6,
            Status::Done => false,
        }
    }

    pub fn move_token(&mut self, count: u8) -> Result<()> {
        if !self.is_valid_move(count) {
            bail!("Not valid move for token {:#?} and move {:?}", self, count);
        }
        match self.status {
            Status::Home => self.status = Status::Running { pos: 1 },
            Status::Running { pos } => {
                let total = pos + count;
                if total <= 51 {
                    self.status = Status::Running { pos: total };
                } else {
                    // Transition to final walk
                    self.status = Status::FinalWalk { pos: 1 };
                    // and then take left steps.
                    self.move_token(total - 52).expect(&format!(
                        "This should not happen with {:?} and move {:?}",
                        self,
                        total - 52
                    ));
                }
            }
            Status::FinalWalk { pos } => {
                let total = pos + count;
                if total == 6 {
                    self.status = Status::Done;
                } else {
                    self.status = Status::FinalWalk { pos: total }
                }
            }
            Status::Done => bail!("Done status does not have any transition"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[should_panic]
    fn panic_for_invalid_token_id(#[values(0, 5)] value: u8) {
        Token::new(Color::Yellow, value);
    }

    #[rstest]
    #[case((Token::new(Color::Red, 1), 6), Token::new(Color::Red, 1).with_staus(Status::Running { pos: 1 }))]
    #[case((Token::new(Color::Red, 1).with_staus(Status::Running { pos: 10 }), 6), Token::new(Color::Red, 1).with_staus(Status::Running { pos: 16 }))]
    #[case((Token::new(Color::Red, 1).with_staus(Status::Running { pos: 49 }), 3), Token::new(Color::Red, 1).with_staus(Status::FinalWalk { pos: 1 }))]
    #[case((Token::new(Color::Red, 1).with_staus(Status::Running { pos: 51 }), 6), Token::new(Color::Red, 1).with_staus(Status::Done))]
    #[case((Token::new(Color::Red, 1).with_staus(Status::FinalWalk { pos: 2 }), 4), Token::new(Color::Red, 1).with_staus(Status::Done))]
    #[case((Token::new(Color::Red, 1).with_staus(Status::FinalWalk { pos: 2 }), 3), Token::new(Color::Red, 1).with_staus(Status::FinalWalk { pos: 5 }))]

    fn test_valid_moves(#[case] input: (Token, u8), #[case] expected: Token) {
        let (mut current, count) = input;
        current.move_token(count).unwrap();
        assert_eq!(current, expected);
    }
}
