use serde::{Deserialize, Serialize};

/// A color of token - we have a board and color in clockwise is Yellow, Blue, Red, Green.
/// For our implementation, we take yello as first quardent.
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Color {
    Yellow,
    Blue,
    Red,
    Green,
}

impl Color {
    /// As yellow is first quardent its offset is 0.
    /// All other color gets +13 offset for each clockwise distance from yellow.
    pub fn pos_offset(&self) -> u32 {
        match &self {
            Color::Yellow => 0,
            Color::Blue => 13,
            Color::Red => 26,
            Color::Green => 39,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pos_offset() {
        assert_eq!(0, Color::Yellow.pos_offset());
        assert_eq!(13, Color::Blue.pos_offset());
        assert_eq!(26, Color::Red.pos_offset());
        assert_eq!(39, Color::Green.pos_offset());
    }
}
