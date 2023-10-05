use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Vector3;
use web_sys::WebGlProgram;
use web_sys::WebGlRenderingContext as GL;

use super::helper::{create_int_js_memory, init_vertex, link_program, uniform_location};
use crate::shaders::fragment::ludo_shader as FS;
use crate::shaders::vertex::ludo_shader as VS;
use anyhow::{anyhow, Result};
pub struct LudoProgram {
    pub program: WebGlProgram,
}

impl LudoProgram {
    pub fn new(gl: &GL) -> Self {
        let program = link_program(gl, VS::LUDO_VERTEX_SHADER, FS::LUDO_FRAGMENT_SHADER)
            .expect("Failed to compile program");
        LudoProgram { program }
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
        let right_x = 10.;
        let left_x = -10.;
        let near_z = -10.;
        let far_z = -30.;
        let top_y = 5.;
        let bottom_y = 4.8;

        let vo = [right_x, top_y, near_z];
        let v1 = [left_x, top_y, near_z];
        let v2 = [left_x, bottom_y, near_z];
        let v3 = [right_x, bottom_y, near_z];
        let v4 = [right_x, bottom_y, far_z];
        let v5 = [right_x, top_y, far_z];
        let v6 = [left_x, top_y, far_z];
        let v7 = [left_x, bottom_y, far_z];
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
            1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., 1., // v0-v5-v6-v1 up(white)
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

        let widht = f32::abs(right_x - left_x);
        let depth = f32::abs(far_z - near_z);

        let inner_sq_left = left_x + 6. * widht / 15.;
        let inner_sq_right = left_x + 9. * widht / 15.;
        let inner_sq_near = near_z - 6. * depth / 15.;
        let inner_sq_far = near_z - 9. * depth / 15.;
        let center_x = left_x + widht / 2.;
        let center_z = near_z - depth / 2.;

        let v8 = [inner_sq_right, top_y + 0.1, inner_sq_near];
        let v9 = [inner_sq_left, top_y + 0.1, inner_sq_near];
        let v10 = [inner_sq_left, top_y + 0.1, inner_sq_far];
        let v11 = [inner_sq_right, top_y + 0.1, inner_sq_far];
        let vmid = [center_x, top_y + 0.1, center_z];
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

        (outer_board, indices, colors)
    }
}
