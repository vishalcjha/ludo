use super::coordinate::Coordinate;

pub trait AntiClockNeighbor {
    fn neighbor(&self) -> Self;
}
impl AntiClockNeighbor for Position {
    fn neighbor(&self) -> Self {
        match self {
            Position::RightFar => Self::RightNear,
            Position::RightNear => Self::LeftNear,
            Position::LeftNear => Self::LeftFar,
            Position::LeftFar => Self::RightFar,
        }
    }
}

pub enum Position {
    RightFar,
    RightNear,
    LeftNear,
    LeftFar,
}

impl Position {
    // this return bottom left cornor for a given color.
    fn begin_x_z_(&self, board_coordinate: &Coordinate) -> (f32, f32) {
        let width = board_coordinate.width();
        let depth = board_coordinate.depth();
        match self {
            Position::LeftNear => (board_coordinate.left, board_coordinate.near),
            Position::RightNear => (
                board_coordinate.right - 6. * width / 15.,
                board_coordinate.near,
            ),
            Position::RightFar => (
                board_coordinate.right - 6. * width / 15.,
                board_coordinate.far + 6. * depth / 15.,
            ),
            Position::LeftFar => (
                board_coordinate.left,
                board_coordinate.far + 6. * depth / 15.,
            ),
        }
    }

    fn play_tile_x_z_(&self, board_coordinate: &Coordinate) -> (f32, f32, u8, u8) {
        let width = board_coordinate.width();
        let depth = board_coordinate.depth();
        match self {
            Position::LeftNear => (
                board_coordinate.left + 6. * width / 15.,
                board_coordinate.near,
                6,
                3,
            ),
            Position::RightNear => (
                board_coordinate.right - 6. * width / 15.,
                board_coordinate.near - 6. * depth / 15.,
                3,
                6,
            ),
            Position::RightFar => (
                board_coordinate.left + 6. * width / 15.,
                board_coordinate.far + 6. * depth / 15.,
                6,
                3,
            ),
            Position::LeftFar => (
                board_coordinate.left,
                board_coordinate.near - 6. * depth / 15.,
                3,
                6,
            ),
        }
    }

    pub(super) fn play_tile_has_color(&self) -> [bool; 18] {
        let mut has_color = [false; 18];
        match self {
            Position::LeftNear => {
                for i in [3, 4, 7, 10, 13, 16] {
                    has_color[i] = true;
                }
            }
            Position::RightNear => {
                for i in [4, 6, 7, 8, 9, 10] {
                    has_color[i] = true;
                }
            }
            Position::RightFar => {
                for i in [1, 4, 7, 10, 13, 14] {
                    has_color[i] = true;
                }
            }
            Position::LeftFar => {
                for i in [7, 8, 9, 10, 11, 13] {
                    has_color[i] = true;
                }
            }
        }
        has_color
    }

    pub(super) fn play_tile_vertices(&self, board_coordinate: &Coordinate) -> Vec<f32> {
        let width = board_coordinate.width();
        let depth = board_coordinate.depth();
        let top = board_coordinate.top + 0.15;
        let padding_factor = 0.015;

        let mut vertices = Vec::<f32>::new();
        let (begin_x, begin_z, row, col) = self.play_tile_x_z_(board_coordinate);
        let cell_widht = width / 15.;
        let cell_depth = depth / 15.;
        for i in 0..row {
            for j in 0..col {
                let begin_x = begin_x + j as f32 * cell_widht;
                let begin_z = begin_z - i as f32 * cell_depth;

                vertices.extend_from_slice(&[
                    begin_x + padding_factor,
                    top,
                    begin_z - padding_factor, // bottom left
                    begin_x - padding_factor + cell_widht,
                    top,
                    begin_z - padding_factor, // bottom right
                    begin_x - padding_factor + cell_widht,
                    top,
                    begin_z - cell_depth + padding_factor, // top right
                    begin_x + padding_factor,
                    top,
                    begin_z - cell_depth + padding_factor,
                ])
            }
        }

        vertices
    }

    // this returns coloured and white part of square in cornor.
    // For color send factor 1 and for white send factor .5.
    pub(super) fn cornor_sq_vetices(&self, board_coordinate: &Coordinate, factor: f32) -> Vec<f32> {
        let width = board_coordinate.width();
        let depth = board_coordinate.depth();
        let top = board_coordinate.top + 0.15;

        let (mut begin_x, mut begin_z) = self.begin_x_z_(board_coordinate);
        let mut cell_count_multiplier = 6.;
        let mut iteration_multiplier = 5.; // how far other strip is.
        if factor != 1. {
            begin_x += width / 15.;
            begin_z -= depth / 15.;
            cell_count_multiplier = 4.;
            iteration_multiplier = 3.5;
        }

        let mut outer_vertices = Vec::<f32>::new();
        let padding_factor = 0.015; // this is to give black line apperance.

        for j in 0..2 {
            let near_z = begin_z - j as f32 * depth * iteration_multiplier / 15.;

            // horizontal outer corner (count 4)
            outer_vertices.extend_from_slice(&[
                (begin_x) + padding_factor,
                top,
                (near_z) - padding_factor, // lower left
                (begin_x + cell_count_multiplier * width / 15.) - padding_factor,
                top,
                (near_z) - padding_factor, // lower right
                (begin_x + cell_count_multiplier * width / 15.) - padding_factor,
                top,
                (near_z - depth * factor / 15.) + padding_factor, // upper right
                (begin_x),
                top,
                (near_z - depth * factor / 15.) + padding_factor, // upper left
            ]);

            // vertical outer corner (count 4)
            let begin_x = begin_x + j as f32 * width * iteration_multiplier / 15.;
            outer_vertices.extend_from_slice(&[
                (begin_x) + padding_factor,
                top,
                (begin_z) - padding_factor, // lower left
                (begin_x + width * factor / 15.) - padding_factor,
                top,
                (begin_z) - padding_factor, // lower right
                (begin_x + width * factor / 15.) - padding_factor,
                top,
                (begin_z - cell_count_multiplier * depth / 15.) + padding_factor, // upper right
                (begin_x) + padding_factor,
                top,
                (begin_z - cell_count_multiplier * depth / 15.) + padding_factor, // upper left
            ]);
        }
        outer_vertices
    }

    pub(super) fn conor_sq_inner_block(&self, board_coordinate: &Coordinate) -> Vec<f32> {
        let mut vertices = Vec::<f32>::new();
        let width = board_coordinate.width();
        let depth = board_coordinate.depth();
        let top = board_coordinate.top + 0.15;
        let (mut begin_x, mut begin_z) = self.begin_x_z_(board_coordinate);
        begin_x += width * 1.5 / 15.;
        begin_z -= depth * 1.5 / 15.;
        let padding_factor = 0.015;
        for i in 0..2 {
            let begin_z = begin_z - i as f32 * depth * 1.5 / 15.;
            for j in 0..2 {
                let begin_x = begin_x + j as f32 * width * 1.5 / 15.;
                vertices.extend_from_slice(&[
                    begin_x + padding_factor,
                    top,
                    begin_z - padding_factor, // left bottom
                    begin_x + width * 1.5 / 15. - padding_factor,
                    top,
                    begin_z - padding_factor, // right bottom,
                    begin_x + width * 1.5 / 15. - padding_factor,
                    top,
                    begin_z - depth * 1.5 / 15. + padding_factor, // right up
                    begin_x + padding_factor,
                    top,
                    begin_z - depth * 1.5 / 15. + padding_factor,
                ]);
            }
        }
        vertices
    }
}
