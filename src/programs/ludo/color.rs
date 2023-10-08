use super::position::AntiClockNeighbor;

#[derive(Debug, Clone)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
}

impl AntiClockNeighbor for Color {
    fn neighbor(&self) -> Self {
        match self {
            Color::Red => Self::Green,
            Color::Green => Self::Yellow,
            Color::Yellow => Self::Blue,
            Color::Blue => Color::Red,
        }
    }
}

impl Color {
    pub fn get_color_tuple(&self) -> [f32; 3] {
        match self {
            Color::Red => [1., 0., 0.],
            Color::Green => [0., 1., 0.],
            Color::Yellow => [1., 1., 0.],
            Color::Blue => [0., 0., 1.],
        }
    }
}
