use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Vector3;
use web_sys::WebGlProgram;
use web_sys::WebGlRenderingContext as GL;

use super::helper::{create_int_js_memory, init_vertex, link_program, uniform_location};
use crate::shaders::fragment::ludo_shader as FS;
use crate::shaders::vertex::ludo_shader as VS;
use anyhow::{anyhow, Result};
use color::Color;

mod color {
    use super::Coordinate;

    pub enum Color {
        Red,
        Green,
        Yellow,
        Blue,
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

        // this return bottom left cornor for a given color.
        fn begin_x_z_(&self, board_coordinate: &Coordinate) -> (f32, f32) {
            let width = board_coordinate.width();
            let depth = board_coordinate.depth();
            match self {
                Color::Yellow => (board_coordinate.left, board_coordinate.near),
                Color::Green => (
                    board_coordinate.right - 6. * width / 15.,
                    board_coordinate.near,
                ),
                Color::Red => (
                    board_coordinate.right - 6. * width / 15.,
                    board_coordinate.far + 6. * depth / 15.,
                ),
                Color::Blue => (
                    board_coordinate.left,
                    board_coordinate.far + 6. * depth / 15.,
                ),
            }
        }

        fn play_tile_x_z_(&self, board_coordinate: &Coordinate) -> (f32, f32, u8, u8) {
            let width = board_coordinate.width();
            let depth = board_coordinate.depth();
            match self {
                Color::Yellow => (
                    board_coordinate.left + 6. * width / 15.,
                    board_coordinate.near,
                    6,
                    3,
                ),
                Color::Green => (
                    board_coordinate.right - 6. * width / 15.,
                    board_coordinate.near - 6. * depth / 15.,
                    3,
                    6,
                ),
                Color::Red => (
                    board_coordinate.left + 6. * width / 15.,
                    board_coordinate.far + 6. * depth / 15.,
                    6,
                    3,
                ),
                Color::Blue => (
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
                Color::Yellow => {
                    for i in [3, 4, 7, 10, 13, 16] {
                        has_color[i] = true;
                    }
                }
                Color::Green => {
                    for i in [4, 6, 7, 8, 9, 10] {
                        has_color[i] = true;
                    }
                }
                Color::Red => {
                    for i in [1, 4, 7, 10, 13, 14] {
                        has_color[i] = true;
                    }
                }
                Color::Blue => {
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
        pub(super) fn cornor_sq_vetices(
            &self,
            board_coordinate: &Coordinate,
            factor: f32,
        ) -> Vec<f32> {
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
}
pub(super) struct Coordinate {
    right: f32,
    left: f32,
    near: f32,
    far: f32,
    top: f32,
    bottom: f32,
}

impl Coordinate {
    fn new() -> Self {
        Coordinate {
            right: 10.,
            left: -10.,
            near: -10.,
            far: -30.,
            top: 5.,
            bottom: 4.8,
        }
    }

    fn width(&self) -> f32 {
        f32::abs(self.right - self.left)
    }

    fn depth(&self) -> f32 {
        f32::abs(self.far - self.near)
    }
}
pub struct LudoProgram {
    pub program: WebGlProgram,
    coorinate: Coordinate,
}

impl LudoProgram {
    pub fn new(gl: &GL) -> Self {
        let program = link_program(gl, VS::LUDO_VERTEX_SHADER, FS::LUDO_FRAGMENT_SHADER)
            .expect("Failed to compile program");
        LudoProgram {
            program,
            coorinate: Coordinate::new(),
        }
    }

    pub fn render(&self, gl: &GL) -> Result<()> {
        gl.clear_color(0., 0., 0., 1.);
        gl.enable(GL::DEPTH_TEST);
        let (board_vertices, indices, colors) = self.get_board_vertices();
        init_vertex(gl, &self.program, "a_Position", &board_vertices)?;
        init_vertex(gl, &self.program, "a_Color", &colors)?;

        assert_eq!(board_vertices.len(), colors.len());

        let u_mvp_matrix = uniform_location(&gl, &self.program, "u_MvpMatrix")?;
        let view_matrix = Matrix4::look_at_rh(
            &Point3::new(0., 20., 15.),
            &Point3::new(0., 5., -40.),
            &Vector3::y(),
        );
        let prespective_matrix = Matrix4::new_perspective(1., std::f32::consts::PI / 4., 1., 100.);
        let mvp_matrix = prespective_matrix * view_matrix;
        gl.uniform_matrix4fv_with_f32_array(Some(&u_mvp_matrix), false, mvp_matrix.as_slice());

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let indices_memory = create_int_js_memory(&indices)?;
        let index_buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_memory,
            GL::STATIC_DRAW,
        );

        gl.draw_elements_with_i32(GL::TRIANGLES, indices.len() as i32, GL::UNSIGNED_SHORT, 0);
        Ok(())
    }

    fn get_board_vertices(&self) -> (Vec<f32>, Vec<u16>, Vec<f32>) {
        let division_count = 15.;
        let Coordinate {
            right,
            left,
            near,
            far,
            top,
            bottom,
        } = self.coorinate;

        let vo = [right, top, near];
        let v1 = [left, top, near];
        let v2 = [left, bottom, near];
        let v3 = [right, bottom, near];
        let v4 = [right, bottom, far];
        let v5 = [right, top, far];
        let v6 = [left, top, far];
        let v7 = [left, bottom, far];
        let mut outer_board: Vec<f32> = Vec::new();

        // Create a ludo board
        //    v6----- v5
        //   /|      /|
        //  v1------v0|
        //  | |     | |
        //  | |v7---|-|v4
        //  |/      |/
        //  v2------v3
        // v0-v1-v2-v3 front
        outer_board.extend_from_slice(&vo);
        outer_board.extend_from_slice(&v1);
        outer_board.extend_from_slice(&v2);
        outer_board.extend_from_slice(&v3);
        // v0-v3-v4-v5 right
        outer_board.extend_from_slice(&vo);
        outer_board.extend_from_slice(&v3);
        outer_board.extend_from_slice(&v4);
        outer_board.extend_from_slice(&v5);
        // v0-v5-v6-v1 up
        outer_board.extend_from_slice(&vo);
        outer_board.extend_from_slice(&v5);
        outer_board.extend_from_slice(&v6);
        outer_board.extend_from_slice(&v1);
        // v1-v6-v7-v2 left
        outer_board.extend_from_slice(&v1);
        outer_board.extend_from_slice(&v6);
        outer_board.extend_from_slice(&v7);
        outer_board.extend_from_slice(&v2);
        // v7-v4-v3-v2 down
        outer_board.extend_from_slice(&v7);
        outer_board.extend_from_slice(&v4);
        outer_board.extend_from_slice(&v3);
        outer_board.extend_from_slice(&v2);
        // v4-v7-v6-v5 back
        outer_board.extend_from_slice(&v4);
        outer_board.extend_from_slice(&v7);
        outer_board.extend_from_slice(&v6);
        outer_board.extend_from_slice(&v5);

        let mut colors: Vec<f32> = vec![
            1.0, 0., 0., 1., 0., 0., 1.0, 0., 0., 1.0, 0., 0., // v0-v1-v2-v3 front(red)
            0., 1.0, 0., 0., 1.0, 0., 0., 1.0, 0., 0., 1.0, 0., // v0-v3-v4-v5 right(green)
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., // v0-v5-v6-v1 up(white)
            0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1., // v1-v6-v7-v2 left
            1.0, 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., // v7-v4-v3-v2 down
            1., 1.0, 0., 1., 1.0, 0., 1., 1.0, 0., 1., 1.0, 0., // v4-v7-v6-v5 back
        ];

        let mut indices = vec![
            0, 1, 2, 0, 2, 3, // front
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // up
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // down
            20, 21, 22, 20, 22, 23,
        ];

        let widht = f32::abs(right - left);
        let depth = f32::abs(far - near);

        let inner_sq_left = left + 6. * widht / 15.;
        let inner_sq_right = left + 9. * widht / 15.;
        let inner_sq_near = near - 6. * depth / 15.;
        let inner_sq_far = near - 9. * depth / 15.;
        let center_x = left + widht / 2.;
        let center_z = near - depth / 2.;

        let v8 = [inner_sq_right, top + 0.2, inner_sq_near];
        let v9 = [inner_sq_left, top + 0.2, inner_sq_near];
        let v10 = [inner_sq_left, top + 0.2, inner_sq_far];
        let v11 = [inner_sq_right, top + 0.2, inner_sq_far];
        let vmid = [center_x, top + 0.15, center_z];
        outer_board.extend_from_slice(&v8);
        outer_board.extend_from_slice(&v9);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v8);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v11);
        outer_board.extend_from_slice(&v11);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v10);
        outer_board.extend_from_slice(&v10);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v9);

        indices.extend_from_slice(&[24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35]);

        colors.extend_from_slice(&[
            1., 1., 0., 1., 1., 0., 1., 1., 0., // yello
            0., 1., 0., 0., 1., 0., 0., 1., 0., // green
            1., 0., 0., 1., 0., 0., 1., 0., 0., // red
            0., 0., 1., 0., 0., 1., 0., 0., 1., // blue
        ]);

        // Now lets make cornor sq first, coloured as well as inner white.
        for color in [Color::Yellow, Color::Green, Color::Red, Color::Blue] {
            for i in 0..2 {
                let factor = 1. / (i as f32 + 1.);

                self.extend_cornor(&mut colors, &mut outer_board, &mut indices, &color, factor);
            }
        }

        for color in [Color::Yellow, Color::Green, Color::Red, Color::Blue] {
            self.extend_for_all_color_conor_sq_inner_block(
                &mut colors,
                &mut outer_board,
                &mut indices,
                &color,
            );
        }

        for color in [Color::Yellow, Color::Green, Color::Red, Color::Blue] {
            self.extend_for_color_tile(&mut colors, &mut outer_board, &mut indices, &color);
        }

        (outer_board, indices, colors)
    }

    fn extend_for_color_tile(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
        color: &Color,
    ) {
        let tile_vertex = color.play_tile_vertices(&self.coorinate);
        let mut tile_vertex = tile_vertex.chunks(3);
        let has_color = color.play_tile_has_color();
        let color_tuple = color.get_color_tuple();
        let white_tuple = [1., 1., 1.];
        for i in 0..18 {
            let one = tile_vertex.next().unwrap();
            let two = tile_vertex.next().unwrap();
            let three = tile_vertex.next().unwrap();
            let four = tile_vertex.next().unwrap();

            let index_begin = vertices.len() as u16 / 3;
            vertices.extend_from_slice(one);
            vertices.extend_from_slice(two);
            vertices.extend_from_slice(three);
            vertices.extend_from_slice(four);

            let selected_color_tuple = if has_color[i] {
                color_tuple
            } else {
                white_tuple
            };
            for _ in 0..4 {
                colors.extend_from_slice(&selected_color_tuple);
            }

            indices.extend_from_slice(&[
                index_begin,
                index_begin + 1,
                index_begin + 2,
                index_begin,
                index_begin + 2,
                index_begin + 3,
            ]);
        }
    }

    fn extend_for_all_color_conor_sq_inner_block(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
        color: &Color,
    ) {
        let four_sq_matrix = color.conor_sq_inner_block(&self.coorinate);
        let mut four_sq_matrix = four_sq_matrix.chunks(3);
        for _ in 0..4 {
            let one = four_sq_matrix.next().unwrap();
            let two = four_sq_matrix.next().unwrap();
            let three = four_sq_matrix.next().unwrap();
            let four = four_sq_matrix.next().unwrap();

            let index_begin = vertices.len() as u16 / 3;
            vertices.extend_from_slice(one);
            vertices.extend_from_slice(two);
            vertices.extend_from_slice(three);
            vertices.extend_from_slice(four);

            let color_tuple = color.get_color_tuple();
            for _ in 0..4 {
                colors.extend_from_slice(&color_tuple);
            }

            indices.extend_from_slice(&[
                index_begin,
                index_begin + 1,
                index_begin + 2,
                index_begin,
                index_begin + 2,
                index_begin + 3,
            ]);
        }
    }

    fn extend_cornor(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
        color: &Color,
        factor: f32,
    ) {
        let begin_index = vertices.len() as u16 / 3;
        let cornor_sq_vertices = color.cornor_sq_vetices(&self.coorinate, factor);
        vertices.extend_from_slice(&cornor_sq_vertices);
        let color_tuple = if factor == 1. {
            color.get_color_tuple()
        } else {
            [1., 1., 1.]
        };
        for _ in 0..16 {
            colors.extend_from_slice(&color_tuple);
        }

        for i in 0..4 {
            let begin_index = (i * 4) + begin_index;
            indices.extend_from_slice(&[
                begin_index,
                begin_index + 1,
                begin_index + 2,
                begin_index,
                begin_index + 2,
                begin_index + 3,
            ]);
        }
    }
}
