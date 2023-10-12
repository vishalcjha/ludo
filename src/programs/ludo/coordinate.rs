#[derive(Debug, Clone)]
pub struct Coordinate {
    pub right: f32,
    pub left: f32,
    pub near: f32,
    pub far: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Coordinate {
    pub fn for_board() -> Self {
        Coordinate {
            right: 10.,
            left: -10.,
            near: -10.,
            far: -30.,
            top: 5.,
            bottom: 4.8,
        }
    }

    pub fn for_dice() -> Self {
        Coordinate {
            right: 1.,
            left: -1.,
            near: -12.,
            far: -14.,
            top: 7.,
            bottom: 5.,
        }
    }

    pub fn width(&self) -> f32 {
        f32::abs(self.right - self.left)
    }

    pub fn depth(&self) -> f32 {
        f32::abs(self.far - self.near)
    }

    pub fn height(&self) -> f32 {
        f32::abs(self.top - self.bottom)
    }
}
