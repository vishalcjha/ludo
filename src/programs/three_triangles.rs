use nalgebra::{Matrix4, Point3, Vector3};
use web_sys::{WebGlProgram, WebGlRenderingContext as GL};

use super::helper::{attribute_location, create_js_memory, link_program, uniform_location};
use crate::shaders::fragment::three_triangles as f_shader;
use crate::shaders::vertex::three_triangles as v_shader;
use anyhow::{anyhow, Result};

const FLOAT_SIZE: i32 = std::mem::size_of::<f32>() as i32;
pub struct ThreeTriangle {
    pub program: WebGlProgram,
}

impl ThreeTriangle {
    pub fn new(gl: &GL) -> Self {
        let program = link_program(
            gl,
            v_shader::THREE_TRIANGLE_SOURCE,
            f_shader::THREE_TRIANGLE_SOURCE,
        )
        .expect("Failed to create web gl program");
        ThreeTriangle { program }
    }

    pub fn run(&self, gl: &GL) -> Result<()> {
        let vertex_count = self.init_vertex_buffer(gl)?;

        gl.clear_color(0., 0., 0., 1.);
        let u_view_matrix = uniform_location(gl, &self.program, "u_ViewMatrix")?;
        let view_matrix = Matrix4::look_at_rh(
            &Point3::new(0., 0., 5.),
            &Point3::new(0., 0., -100.),
            &Vector3::new(0., 1., 0.),
        );

        gl.uniform_matrix4fv_with_f32_array(Some(&u_view_matrix), false, view_matrix.as_slice());

        let perspective_matrix = Matrix4::new_perspective(1., std::f32::consts::PI / 4., 1., 200.);

        let u_model_matrix = uniform_location(gl, &self.program, "u_ProjMatrix")?;
        gl.uniform_matrix4fv_with_f32_array(
            Some(&u_model_matrix),
            false,
            perspective_matrix.as_slice(),
        );
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.draw_arrays(GL::TRIANGLES, 0, vertex_count as i32);
        Ok(())
    }

    fn init_vertex_buffer(&self, gl: &GL) -> Result<u32> {
        let vertex_color_data: [f32; 108] = [
            // Three triangles on the right side
            0.75, 1.0, -4.0, 0.4, 1.0, 0.4, // The back green triangle
            0.25, -1.0, -4.0, 0.4, 1.0, 0.4, //
            1.25, -1.0, -4.0, 1.0, 0.4, 0.4, //
            0.75, 1.0, -2.0, 1.0, 1.0, 0.4, //
            0.25, -1.0, -2.0, 1.0, 1.0, 0.4, //
            1.25, -1.0, -2.0, 1.0, 0.4, 0.4, //
            0.75, 1.0, 0.0, 0.4, 0.4, 1.0, //
            0.25, -1.0, 0.0, 0.4, 0.4, 1.0, //
            1.25, -1.0, 0.0, 1.0, 0.4, 0.4, //
            // Three triangles on the left side
            -0.75, 1.0, -4.0, 0.4, 1.0, 0.4, //
            -1.25, -1.0, -4.0, 0.4, 1.0, 0.4, //
            -0.25, -1.0, -4.0, 1.0, 0.4, 0.4, //
            -0.75, 1.0, -2.0, 1.0, 1.0, 0.4, //
            -1.25, -1.0, -2.0, 1.0, 1.0, 0.4, //
            -0.25, -1.0, -2.0, 1.0, 0.4, 0.4, //
            -0.75, 1.0, 0.0, 0.4, 0.4, 1.0, //
            -1.25, -1.0, 0.0, 0.4, 0.4, 1.0, //
            -0.25, -1.0, 0.0, 1.0, 0.4, 0.4,
        ];

        let buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let js_memory = create_js_memory(&vertex_color_data)?;
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_memory, GL::STATIC_DRAW);

        let a_position = attribute_location(&self.program, gl, "a_Position")?;
        gl.vertex_attrib_pointer_with_i32(a_position, 3, GL::FLOAT, false, FLOAT_SIZE * 6, 0);
        gl.enable_vertex_attrib_array(a_position);

        let a_color = attribute_location(&self.program, gl, "a_Color")?;
        gl.vertex_attrib_pointer_with_i32(
            a_color,
            3,
            GL::FLOAT,
            false,
            FLOAT_SIZE * 6,
            FLOAT_SIZE * 3,
        );
        gl.enable_vertex_attrib_array(a_color);
        Ok(18)
    }
}
