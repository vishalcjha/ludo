use std::{cell::RefCell, rc::Rc};

use browser::{canvas, context, height, spawn_local, window};
use programs::cube_program::CubeProgram;
use wasm_bindgen::prelude::*;

mod browser;
mod engine;
mod programs;
mod shaders;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    set_canvas_size().unwrap();

    spawn_local(async {
        let gl = context().unwrap();
        let cube_program = CubeProgram::new(&gl);
        gl.use_program(Some(&cube_program.program));
        let mut angle = 45.0;
        let animation_loop = Rc::new(RefCell::new(None));
        let animation_loop_cloned = animation_loop.clone();
        *animation_loop_cloned.borrow_mut() = Some(Closure::new(move || {
            angle = angle % 360.;
            let gl = context().unwrap();
            if let Err(err) = cube_program.render(&gl) {
                web_sys::console::log_1(&format!("Failed with error {:#?}", err).into());
            }

            if angle > 90.0 {
                request_animation_frame(animation_loop.borrow().as_ref().unwrap());
            }
        }));
        request_animation_frame(animation_loop_cloned.borrow().as_ref().unwrap());
    });

    Ok(())
}

fn set_canvas_size() -> anyhow::Result<u32> {
    let canvas = canvas().unwrap();

    let height = height()?;
    canvas.set_height(height);
    canvas.set_width(height);

    Ok(height)
}
