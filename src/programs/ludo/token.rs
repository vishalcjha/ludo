use super::{color::Color, coordinate::Coordinate};

pub(super) struct Token<'a> {
    color: Color,
    position: TokenPosition,
    board_coordinate: &'a Coordinate,
}

struct TokenPosition {
    x: f32,
    y: f32,
    z: f32,
}
