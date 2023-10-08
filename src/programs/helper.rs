use anyhow::{anyhow, Result};
use js_sys::{Float32Array, Uint16Array, WebAssembly};
use std::convert::TryFrom;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};
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

pub fn attribute_location(
    program: &WebGlProgram,
    gl: &WebGlRenderingContext,
    location_name: impl AsRef<str>,
) -> Result<u32> {
    u32::try_from(gl.get_attrib_location(&program, location_name.as_ref())).map_err(|err| {
        anyhow!(format!(
            "Failed to get attribute for localtion {:?} with error {:#?}",
            location_name.as_ref(),
            err
        ))
    })
}

pub fn create_wasam_memory() -> Result<JsValue> {
    Ok(wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .map_err(|err| anyhow!(format!("Failed to create wasam memory {:#?}", err)))?
        .buffer())
}

pub fn create_js_memory(data: &[f32]) -> Result<Float32Array> {
    let memory = create_wasam_memory()?;
    let start_location = data.as_ptr() as u32 / 4;
    let js_memory =
        Float32Array::new(&memory).subarray(start_location, start_location + data.len() as u32);
    Ok(js_memory)
}

pub fn create_int_js_memory(data: &[u16]) -> Result<Uint16Array> {
    let memory = create_wasam_memory()?;
    let start_location = data.as_ptr() as u32 / 2;
    let js_memory =
        Uint16Array::new(&memory).subarray(start_location, start_location + data.len() as u32);
    Ok(js_memory)
}

pub fn uniform_location(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    uniform_name: impl AsRef<str>,
) -> Result<WebGlUniformLocation> {
    gl.get_uniform_location(&program, uniform_name.as_ref())
        .ok_or_else(|| anyhow!(format!("Failed to get uniform {:?}", uniform_name.as_ref())))
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

pub fn init_vertex(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    attribute_name: impl AsRef<str>,
    data: &[f32],
) -> Result<()> {
    let buffer = gl
        .create_buffer()
        .ok_or_else(|| anyhow!("Failed to create buffer"))?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    let data = create_js_memory(data)?;
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &data,
        WebGlRenderingContext::STATIC_DRAW,
    );
    let attribute = attribute_location(program, &gl, attribute_name.as_ref())?;
    gl.vertex_attrib_pointer_with_i32(attribute, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(attribute);
    // gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
    Ok(())
}
