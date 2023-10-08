use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Vector3;
use web_sys::WebGlProgram;
use web_sys::WebGlRenderingContext as GL;

use crate::programs::helper::{create_int_js_memory, init_vertex, link_program, uniform_location};
use crate::shaders::fragment::ludo_shader as FS;
use crate::shaders::vertex::ludo_shader as VS;
use anyhow::{anyhow, Result};

use super::color::Color;
use super::coordinate::Coordinate;
use super::position::{AntiClockNeighbor, Position};

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

    pub fn render(&self, gl: &GL, left_near_color: Color) -> Result<()> {
        gl.clear_color(0., 0., 0., 1.);
        gl.enable(GL::DEPTH_TEST);
        let (board_vertices, indices, colors) = self.get_board_vertices(&left_near_color);
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

    fn get_board_vertices(&self, left_near_color: &Color) -> (Vec<f32>, Vec<u16>, Vec<f32>) {
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

        (outer_board, indices, colors)
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
