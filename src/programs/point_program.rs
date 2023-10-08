#![allow(dead_code)]
use js_sys::{Float32Array, WebAssembly};
use nalgebra::Vector3;
use std::convert::TryFrom;
use std::f32::consts;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::shaders::{fragment::fpoint_shader, vertex::vpoint_shader};

use super::helper::link_program;
use anyhow::{anyhow, Result};

pub struct PointProgram {
    pub program: WebGlProgram,
}

impl PointProgram {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = link_program(
            gl,
            vpoint_shader::VERTEX_SHADER,
            fpoint_shader::POINT_SHADER,
        )
        .unwrap();
        PointProgram { program }
    }

    pub fn assign_position(
        &self,
        gl: &WebGlRenderingContext,
        position: [f32; 6],
        angle: f32,
    ) -> Result<()> {
        let buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let vert_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let position_location = position.as_ptr() as u32 / 4;
        let float_array = Float32Array::new(&vert_buffer)
            .subarray(position_location, position_location + position.len() as u32);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &float_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        let a_position = u32::try_from(gl.get_attrib_location(&self.program, "a_Position"))
            .map_err(|err| anyhow!(format!("Failed to get a_Position with error {:#?}", err)))?;

        gl.vertex_attrib_pointer_with_i32(a_position, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_position);

        let u_translation = gl
            .get_uniform_location(&self.program, "u_xFormMatrix")
            .ok_or_else(|| anyhow!("failed to get translation from vertex shader"))?;
        let rotation_matrix =
            nalgebra::Matrix4::new_rotation(Vector3::z() * consts::PI * angle / 180.)
                .append_translation(&Vector3::new(-0.1, 0., 0.));

        gl.uniform_matrix4fv_with_f32_array(
            Some(&u_translation),
            false,
            rotation_matrix.as_slice(),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts;

    use nalgebra::{Matrix4, Vector3};
    #[test]
    fn get_matric_rotation() {
        let radian_angle = consts::PI * 90. / 180.;
        let rotation_matrix = Matrix4::new_rotation(Vector3::z() * radian_angle);
        let transformed_matrix = rotation_matrix.as_slice();
        let cos_b = radian_angle.cos();
        let sig_b = radian_angle.sin();
        println!("{:?}", rotation_matrix);
        println!("Cos is {} and sin is {}", cos_b, sig_b);
        println!("Transformed matrix is {:?}", transformed_matrix);
    }
}
