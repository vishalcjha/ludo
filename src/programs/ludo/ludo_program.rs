use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Vector3;
use web_sys::WebGlProgram;
use web_sys::WebGlRenderingContext as GL;

use crate::programs::helper::{create_int_js_memory, init_vertex, link_program, uniform_location};
use crate::shaders::fragment::ludo_shader as FS;
use crate::shaders::vertex::ludo_shader as VS;
use anyhow::{anyhow, Result};

use super::board_configuration::BoardConfiguration;
use super::color::Color;
use super::coordinate::Coordinate;
use super::position::{AntiClockNeighbor, Position};

pub struct LudoProgram {
    pub program: WebGlProgram,
    coorinate: Coordinate,
}

pub enum Around {
    X,
    Y,
    Z,
}

impl LudoProgram {
    pub fn new(gl: &GL) -> Self {
        let program = link_program(gl, VS::LUDO_VERTEX_SHADER, FS::LUDO_FRAGMENT_SHADER)
            .expect("Failed to compile program");
        LudoProgram {
            program,
            coorinate: Coordinate::for_board(),
        }
    }

    pub fn render(&self, gl: &GL, left_near_color: Color, angle: f32) -> Result<()> {
        gl.clear_color(0., 0., 0., 1.);
        gl.enable(GL::DEPTH_TEST);
        let BoardConfiguration {
            vertices,
            indices,
            colors,
            start_index_dice,
            end_index_dice,
        } = self.get_board_vertices(&left_near_color);
        init_vertex(gl, &self.program, "a_Position", &vertices)?;
        init_vertex(gl, &self.program, "a_Color", &colors)?;

        assert_eq!(vertices.len(), colors.len());

        let u_mvp_matrix = uniform_location(&gl, &self.program, "u_MvpMatrix")?;
        let view_matrix = Matrix4::look_at_rh(
            &Point3::new(0., 20., 15.),
            &Point3::new(0., 5., -40.),
            &Vector3::y(),
        );
        let prespective_matrix = Matrix4::new_perspective(1., std::f32::consts::PI / 4., 1., 100.);
        let rotation = Matrix4::new_rotation_wrt_point(
            Vector3::y() * std::f32::consts::PI * 45. / 180.,
            Point3::new(0., 0., -20.),
        );
        let mvp_matrix = prespective_matrix * view_matrix * rotation;
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

        gl.draw_elements_with_i32(GL::TRIANGLES, start_index_dice, GL::UNSIGNED_SHORT, 0);

        // rotate dice
        let rotation = Matrix4::new_rotation_wrt_point(
            Vector3::y() * std::f32::consts::PI * angle / 180.,
            Point3::new(0., 0., -20.),
        );
        let mvp_matrix = prespective_matrix * view_matrix * rotation;
        gl.uniform_matrix4fv_with_f32_array(Some(&u_mvp_matrix), false, mvp_matrix.as_slice());
        gl.draw_elements_with_i32(
            GL::TRIANGLES,
            end_index_dice - start_index_dice,
            GL::UNSIGNED_SHORT,
            2 * start_index_dice,
        );
        Ok(())
    }

    fn get_board_vertices(&self, left_near_color: &Color) -> BoardConfiguration {
        let mut outer_board: Vec<f32> = Vec::new();
        self.extend_with_cube_vertices(self.coorinate.clone(), &mut outer_board);

        let mut colors: Vec<f32> = vec![
            1.0, 1., 0., 1., 1., 0., 1.0, 1., 0., 1.0, 1., 0., // v0-v1-v2-v3 front(yello)
            0., 1.0, 0., 0., 1.0, 0., 0., 1.0, 0., 0., 1.0, 0., // v0-v3-v4-v5 right(green)
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., // v0-v5-v6-v1 up(black)
            0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1., // v1-v6-v7-v2 left(blue)
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., // v7-v4-v3-v2 down
            1., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., // v4-v7-v6-v5 back (red)
        ];

        let mut indices = vec![
            0, 1, 2, 0, 2, 3, // front
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // up
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // down
            20, 21, 22, 20, 22, 23,
        ];

        let Coordinate {
            right,
            left,
            near,
            far,
            top,
            ..
        } = &self.coorinate;
        let widht = f32::abs(right - left);
        let depth = f32::abs(far - near);

        let inner_sq_left = left + 6. * widht / 15.;
        let inner_sq_right = left + 9. * widht / 15.;
        let inner_sq_near = near - 6. * depth / 15.;
        let inner_sq_far = near - 9. * depth / 15.;
        let center_x = left + widht / 2.;
        let center_z = near - depth / 2.;

        let v8 = [inner_sq_right, top + 0.15, inner_sq_near];
        let v9 = [inner_sq_left, top + 0.15, inner_sq_near];
        let v10 = [inner_sq_left, top + 0.15, inner_sq_far];
        let v11 = [inner_sq_right, top + 0.15, inner_sq_far];
        let vmid = [center_x, top + 0.15, center_z];
        // add inner block in anti clock orientation.
        outer_board.extend_from_slice(&v8);
        outer_board.extend_from_slice(&v9);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v10);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v9);
        outer_board.extend_from_slice(&v11);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v10);
        outer_board.extend_from_slice(&v8);
        outer_board.extend_from_slice(&vmid);
        outer_board.extend_from_slice(&v11);

        indices.extend_from_slice(&[24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35]);

        let position_color_map = [
            (Position::LeftNear, left_near_color.clone()),
            (Position::LeftFar, left_near_color.neighbor().clone()),
            (
                Position::RightFar,
                left_near_color.neighbor().neighbor().clone(),
            ),
            (
                Position::RightNear,
                left_near_color.neighbor().neighbor().neighbor().clone(),
            ),
        ];

        for position_color in &position_color_map {
            let color = position_color.1.get_color_tuple();
            for _ in 0..3 {
                colors.extend_from_slice(&color);
            }
        }

        // Now lets make cornor sq first, coloured as well as inner white.
        for position_color in &position_color_map {
            for i in 0..2 {
                let factor = 1. / (i as f32 + 1.);

                self.extend_cornor(
                    &mut colors,
                    &mut outer_board,
                    &mut indices,
                    &position_color.0,
                    &position_color.1,
                    factor,
                );
            }
        }

        for position_color in &position_color_map {
            self.extend_for_all_color_conor_sq_inner_block(
                &mut colors,
                &mut outer_board,
                &mut indices,
                &position_color.0,
                &position_color.1,
            );
        }

        for position_color in &position_color_map {
            self.extend_for_color_tile(
                &mut colors,
                &mut outer_board,
                &mut indices,
                &position_color.0,
                &position_color.1,
            );
        }

        let mut board_configuraton = BoardConfiguration {
            vertices: outer_board,
            colors,
            start_index_dice: indices.len() as i32,
            end_index_dice: indices.len() as i32,
            indices,
        };

        self.append_with_dice(
            &mut board_configuraton.colors,
            &mut board_configuraton.vertices,
            &mut board_configuraton.indices,
        );
        board_configuraton.end_index_dice = board_configuraton.indices.len() as i32;
        board_configuraton
    }

    fn extend_with_cube_vertices(&self, coorinate: Coordinate, vertices: &mut Vec<f32>) {
        let Coordinate {
            right,
            left,
            near,
            far,
            top,
            bottom,
        } = coorinate;
        let vo = [right, top, near];
        let v1 = [left, top, near];
        let v2 = [left, bottom, near];
        let v3 = [right, bottom, near];
        let v4 = [right, bottom, far];
        let v5 = [right, top, far];
        let v6 = [left, top, far];
        let v7 = [left, bottom, far];

        // Create a ludo board
        //    v6----- v5
        //   /|      /|
        //  v1------v0|
        //  | |     | |
        //  | |v7---|-|v4
        //  |/      |/
        //  v2------v3
        // v0-v1-v2-v3 front
        vertices.extend_from_slice(&vo);
        vertices.extend_from_slice(&v1);
        vertices.extend_from_slice(&v2);
        vertices.extend_from_slice(&v3);
        // v0-v3-v4-v5 right
        vertices.extend_from_slice(&vo);
        vertices.extend_from_slice(&v3);
        vertices.extend_from_slice(&v4);
        vertices.extend_from_slice(&v5);
        // v0-v5-v6-v1 up
        vertices.extend_from_slice(&vo);
        vertices.extend_from_slice(&v5);
        vertices.extend_from_slice(&v6);
        vertices.extend_from_slice(&v1);
        // v1-v6-v7-v2 left
        vertices.extend_from_slice(&v1);
        vertices.extend_from_slice(&v6);
        vertices.extend_from_slice(&v7);
        vertices.extend_from_slice(&v2);
        // v7-v4-v3-v2 down
        vertices.extend_from_slice(&v7);
        vertices.extend_from_slice(&v4);
        vertices.extend_from_slice(&v3);
        vertices.extend_from_slice(&v2);
        // v4-v7-v6-v5 back
        vertices.extend_from_slice(&v4);
        vertices.extend_from_slice(&v7);
        vertices.extend_from_slice(&v6);
        vertices.extend_from_slice(&v5);
    }
    fn extend_for_color_tile(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
        position: &Position,
        color: &Color,
    ) {
        let tile_vertex = position.play_tile_vertices(&self.coorinate);
        let mut tile_vertex = tile_vertex.chunks(3);
        let has_color = position.play_tile_has_color();
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

    fn append_with_dice(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
    ) {
        let mut begin = vertices.len() as u16;
        self.extend_with_cube_vertices(Coordinate::for_dice(), vertices);
        let dice_coordinate = Coordinate::for_dice();
        let Coordinate {
            right,
            left,
            near,
            far,
            top,
            bottom,
        } = dice_coordinate;
        assert_eq!(vertices.len() as u16, begin + 3 * 24);
        // Create a ludo board
        //    v6----- v5
        //   /|      /|
        //  v1------v0|
        //  | |     | |
        //  | |v7---|-|v4
        //  |/      |/
        //  v2------v3
        // v0-v1-v2-v3 front

        colors.extend_from_slice(&[
            0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95,
            0.95, // v0-v1-v2-v3 front(white)
            0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98,
            0.98, // v0-v3-v4-v5 right(white)
            0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92,
            0.92, // v0-v5-v6-v1 up(white)
            0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98,
            0.98, // v1-v6-v7-v2 left(white)
            0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92, 0.92,
            0.92, // v7-v4-v3-v2 donw (white)
            0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95,
            0.95, // v4-v7-v6-v5 back (white)
        ]);

        begin = begin / 3;
        indices.extend_from_slice(&[
            begin + 0,
            begin + 1,
            begin + 2,
            begin + 0,
            begin + 2,
            begin + 3, // front
            begin + 4,
            begin + 5,
            begin + 6,
            begin + 4,
            begin + 6,
            begin + 7, // right
            begin + 8,
            begin + 9,
            begin + 10,
            begin + 8,
            begin + 10,
            begin + 11, // up
            begin + 12,
            begin + 13,
            begin + 14,
            begin + 12,
            begin + 14,
            begin + 15, // left
            begin + 16,
            begin + 17,
            begin + 18,
            begin + 16,
            begin + 18,
            begin + 19, // down
            begin + 20,
            begin + 21,
            begin + 22,
            begin + 20,
            begin + 22,
            begin + 23,
        ]);

        begin = vertices.len() as u16 / 3;
        let x_mid = (right + left) / 2.;
        let y_mid = (top + bottom) / 2.;
        let z_mid = (near + far) / 2.;

        let mut extend_around = |x: f32, y: f32, z: f32, around: Around| {
            match around {
                Around::X => {
                    vertices.extend_from_slice(&[x, y, z - 0.15, x + 0.15, y, z, x - 0.15, y, z]);
                }
                Around::Y => {
                    vertices.extend_from_slice(&[x, y, z - 0.15, x + -0.15, y, z, x + 0.15, y, z]);
                }
                Around::Z => {
                    vertices.extend_from_slice(&[x, y + 0.15, z, x + 0.15, y, z, x - 0.15, y, z]);
                }
            };
        };
        // do 1
        extend_around(x_mid, y_mid, near + 0.015, Around::Z);

        // do 2
        extend_around(
            left,
            y_mid,
            near - (dice_coordinate.depth() / 4.),
            Around::X,
        );
        extend_around(left, y_mid, far + (dice_coordinate.depth() / 4.), Around::X);

        // do 3
        extend_around(left + dice_coordinate.width() / 4., y_mid, far, Around::X);
        extend_around(
            left + 2. * dice_coordinate.width() / 4.,
            y_mid,
            far,
            Around::X,
        );
        extend_around(
            left + 3. * dice_coordinate.width() / 4.,
            y_mid,
            far,
            Around::X,
        );

        // do 4
        extend_around(
            right,
            top - (dice_coordinate.depth() / 4.),
            near - (dice_coordinate.depth() / 4.),
            Around::X,
        );
        extend_around(
            right,
            top - (dice_coordinate.depth() / 4.),
            far + (dice_coordinate.depth() / 4.),
            Around::X,
        );
        extend_around(
            right,
            bottom + (dice_coordinate.depth() / 4.),
            near - (dice_coordinate.depth() / 4.),
            Around::X,
        );
        extend_around(
            right,
            bottom + (dice_coordinate.depth() / 4.),
            far + (dice_coordinate.depth() / 4.),
            Around::X,
        );

        // do six on top
        extend_around(
            right - (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            right - (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (2. * dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            right - (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (3. * dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            left + (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            left + (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (2. * dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            left + (dice_coordinate.width() / 4.),
            top + 0.015,
            near - (3. * dice_coordinate.depth() / 4.),
            Around::Y,
        );

        // do 5
        extend_around(
            right - (dice_coordinate.width() / 4.),
            bottom - 0.015,
            near - (dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            right - (dice_coordinate.width() / 4.),
            bottom - 0.015,
            near - (3. * dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            right - (dice_coordinate.width() / 2.),
            bottom - 0.015,
            near - (dice_coordinate.depth() / 2.),
            Around::Y,
        );
        extend_around(
            left + (dice_coordinate.width() / 4.),
            bottom - 0.015,
            near - (dice_coordinate.depth() / 4.),
            Around::Y,
        );
        extend_around(
            left + (dice_coordinate.width() / 4.),
            bottom - 0.015,
            near - (3. * dice_coordinate.depth() / 4.),
            Around::Y,
        );

        colors.extend_from_slice(&[0.; 21 * 3 * 3]);
        indices.append(&mut (begin..begin + 21 * 3).collect());
    }

    fn extend_for_all_color_conor_sq_inner_block(
        &self,
        colors: &mut Vec<f32>,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u16>,
        position: &Position,
        color: &Color,
    ) {
        let four_sq_matrix = position.conor_sq_inner_block(&self.coorinate);
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
        position: &Position,
        color: &Color,
        factor: f32,
    ) {
        let begin_index = vertices.len() as u16 / 3;
        let cornor_sq_vertices = position.cornor_sq_vetices(&self.coorinate, factor);
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
