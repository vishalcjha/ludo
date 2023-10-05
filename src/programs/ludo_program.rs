use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::helper::link_program;
use crate::shaders::fragment::ludo_shader as LS;
use crate::shaders::vertex::ludo_shader as VS;
use anyhow::Result;
pub struct LudoProgram {
    pub program: WebGlProgram,
}

impl LudoProgram {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = link_program(gl, VS::LUDO_VERTEX_SHADER, LS::LUDO_FRAGMENT_SHADER)
            .expect("Failed to compile program");
        LudoProgram { program }
    }

    pub fn render(&self, _gl: &WebGlRenderingContext) -> Result<()> {
        Ok(())
    }
}
