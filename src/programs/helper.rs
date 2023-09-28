use anyhow::{anyhow, Result};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};
pub fn link_program(
    gl: &WebGlRenderingContext,
    vetrex_src: &str,
    fragment_src: &str,
) -> Result<WebGlProgram> {
    let program = gl
        .create_program()
        .ok_or_else(|| anyhow!("Failed to create program"))?;

    let vertex_shader = compile_shader(gl, vetrex_src, WebGlRenderingContext::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(gl, fragment_src, WebGlRenderingContext::FRAGMENT_SHADER)?;
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(anyhow!(gl.get_program_info_log(&program).unwrap_or_else(
            || "Failed to get program info log".to_owned()
        )))
    }
}
fn compile_shader(gl: &WebGlRenderingContext, src: &str, shader_type: u32) -> Result<WebGlShader> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| anyhow!("failed to create shader"))?;
    gl.shader_source(&shader, src);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(anyhow!(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Failed to get failure log".to_owned())))
    }
}
