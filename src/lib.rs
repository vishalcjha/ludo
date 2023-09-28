use browser::context;
use programs::point_program::PointProgram;
use wasm_bindgen::prelude::*;
use web_sys::{console, WebGlRenderingContext};

mod browser;
mod programs;
mod shaders;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let gl = context().unwrap();
    let point_program = PointProgram::new(&gl);
    gl.use_program(Some(&point_program.program));
    gl.clear_color(0., 0., 0., 1.);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    gl.draw_arrays(WebGlRenderingContext::POINTS, 0, 1);
    Ok(())
}
