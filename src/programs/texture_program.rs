use js_sys::{Float32Array, WebAssembly};
use wasm_bindgen::JsCast;
use web_sys::{HtmlImageElement, WebGlProgram, WebGlRenderingContext};

use crate::shaders::vertex::texture_shader as vertex;
use crate::{browser, shaders::fragment::texture_shader as fragment};
use anyhow::{anyhow, Ok, Result};

use super::helper::link_program;
use std::convert::TryFrom;
use web_sys::WebGlRenderingContext as GL;

const FLOAT_SIZE: i32 = std::mem::size_of::<f32>() as i32;
pub struct TextureProgram {
    pub program: WebGlProgram,
}

impl TextureProgram {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = link_program(
            gl,
            vertex::TEXTURE_VERTEX_SHADER,
            fragment::TEXTURE_FRAGMENT_SHADER,
        )
        .unwrap();
        TextureProgram { program }
    }
    pub fn assing_attributes(&self, gl: &WebGlRenderingContext) -> Result<u32> {
        let buffer_data: [f32; 16] = [
            // vertex cood, texture coord
            -0.5, 0.5, 0., 1., -0.5, -0.5, 0., 0., 0.5, 0.5, 1., 1., 0.5, -0.5, 1., 0.,
        ];
        let buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow!("Failed to create buffer"))?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        let gl_memory = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let buffer_data_location = buffer_data.as_ptr() as u32 / 4;
        let js_memory = Float32Array::new(&gl_memory).subarray(
            buffer_data_location,
            buffer_data_location + buffer_data.len() as u32,
        );
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_memory, GL::STATIC_DRAW);

        let a_position = self.attribute_location(gl, "a_Position")?;
        gl.vertex_attrib_pointer_with_i32(a_position, 2, GL::FLOAT, false, FLOAT_SIZE * 4, 0);
        gl.enable_vertex_attrib_array(a_position);

        let a_tex_coord = self.attribute_location(gl, "a_TexCoord")?;
        gl.vertex_attrib_pointer_with_i32(
            a_tex_coord,
            2,
            GL::FLOAT,
            false,
            FLOAT_SIZE * 4,
            FLOAT_SIZE * 2,
        );
        gl.enable_vertex_attrib_array(a_tex_coord);

        // web_sys::console::log_1(&format!("Size of float is {:?}", FLOAT_SIZE).into());

        Ok(4)
    }

    pub fn init_texture(
        &self,
        gl: &WebGlRenderingContext,
        image: &HtmlImageElement,
        count: u32,
    ) -> Result<()> {
        let texture = gl
            .create_texture()
            .ok_or_else(|| anyhow!("Failed to create texture"))?;
        let sampler_uniform = gl
            .get_uniform_location(&self.program, "u_Sampler")
            .ok_or_else(|| anyhow!("Failed to get Sampler Uniform"))?;

        gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_image_2d_with_u32_and_u32_and_image(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            image,
        )
        .map_err(|err| {
            anyhow!(format!(
                "Failed to attach image to texture with error {:#?}",
                err
            ))
        })?;

        gl.uniform1i(Some(&sampler_uniform), 0);
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.draw_arrays(GL::TRIANGLE_STRIP, 0, count as i32);

        web_sys::console::log_1(&"Finished texture mapping successfully".into());
        Ok(())
    }

    fn attribute_location(
        &self,
        gl: &WebGlRenderingContext,
        location_name: impl AsRef<str>,
    ) -> Result<u32> {
        u32::try_from(gl.get_attrib_location(&self.program, location_name.as_ref())).map_err(
            |err| {
                anyhow!(format!(
                    "Failed to get attribute for localtion {:?} with error {:#?}",
                    location_name.as_ref(),
                    err
                ))
            },
        )
    }
}
