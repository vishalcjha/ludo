use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::shaders::{fragment::fpoint_shader, vertex::vpoint_shader};

use super::helper::link_program;

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
}
