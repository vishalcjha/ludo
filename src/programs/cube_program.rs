use nalgebra::{Matrix4, Point3, Vector3};
use web_sys::{WebGlProgram, WebGlRenderingContext as GL};

use super::helper::{
    attribute_location, create_int_js_memory, create_js_memory, link_program, uniform_location,
};
use crate::shaders::fragment::cube_shader as FCS;
use crate::shaders::vertex::cube_shader as VCS;
use anyhow::{anyhow, Result};

pub struct CubeProgram {
    pub program: WebGlProgram,
}

impl CubeProgram {
    pub fn new(gl: &GL) -> Self {
        let program = link_program(&gl, VCS::VERTEX_CUBE_SHADER, FCS::FRAGMENT_CUBE_SHADER)
            .expect("Fail to link Cube program");
        CubeProgram { program }
    }

    pub fn render(&self, gl: &GL) -> Result<()> {
        let element_count = self.make_vertex_context(gl)? as i32;
        gl.clear_color(0., 0., 0., 1.);
        gl.enable(GL::DEPTH_TEST);

        let u_mvp_matrix = uniform_location(&gl, &self.program, "u_MvpMatrix")?;
        let prespective_matrix = Matrix4::new_perspective(1., std::f32::consts::PI / 6., 1., 100.);
        let view_matrix = Matrix4::look_at_rh(
            &Point3::new(3., 3., 7.),
            &Point3::new(0., 0., 0.),
            &Vector3::y(),
        );
        let mvp_matrix = prespective_matrix * view_matrix;
        gl.uniform_matrix4fv_with_f32_array(Some(&u_mvp_matrix), false, mvp_matrix.as_slice());

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        gl.draw_elements_with_i32(GL::TRIANGLE_STRIP, element_count, GL::UNSIGNED_BYTE, 0);
        Ok(())
    }

    fn make_vertex_context(&self, gl: &GL) -> Result<u32> {
        let vetext_array: [f32; 72] = [
            1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0,
            1.0, // v0-v1-v2-v3 front
            1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
            -1.0, // v0-v3-v4-v5 right
            1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, // v0-v5-v6-v1 up
            -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,
            1.0, // v1-v6-v7-v2 left
            -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0,
            1.0, // v7-v4-v3-v2 down
            1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0,
            -1.0, // v4-v7-v6-v5 back
        ];

        let colors: [f32; 72] = [
            0.4, 0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4,
            1.0, // v0-v1-v2-v3 front(blue)
            0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4, 1.0,
            0.4, // v0-v3-v4-v5 right(green)
            1.0, 0.4, 0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4, 1.0, 0.4, 0.4, // v0-v5-v6-v1 up(red)
            1.0, 1.0, 0.4, 1.0, 1.0, 0.4, 1.0, 1.0, 0.4, 1.0, 1.0, 0.4, // v1-v6-v7-v2 left
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, // v7-v4-v3-v2 down
            0.4, 1.0, 1.0, 0.4, 1.0, 1.0, 0.4, 1.0, 1.0, 0.4, 1.0, 1.0, // v4-v7-v6-v5 back
        ];

        let indices: [u16; 36] = [
            0, 1, 2, 0, 2, 3, // front
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // up
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // down
            20, 21, 22, 20, 22, 23,
        ];

        let vetex_memory = create_js_memory(&vetext_array)?;
        let color_memory = create_js_memory(&colors)?;
        let indice_memory = create_int_js_memory(&indices)?;

        let vertex_buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vetex_memory, GL::STATIC_DRAW);

        let a_attribute = attribute_location(&self.program, &gl, "a_Position")?;
        gl.vertex_attrib_pointer_with_i32(a_attribute, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_attribute);

        let color_buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_memory, GL::STATIC_DRAW);

        let a_color = attribute_location(&self.program, &gl, "a_Color")?;
        gl.vertex_attrib_pointer_with_i32(a_color, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_color);

        let index_buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indice_memory,
            GL::STATIC_DRAW,
        );

        Ok(indices.len() as u32)
    }
}
