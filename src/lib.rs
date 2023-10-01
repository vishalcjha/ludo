use std::{cell::RefCell, rc::Rc};

use browser::{context, spawn_local, window};
use engine::load_image;
use programs::{texture_program::TextureProgram, three_triangles::ThreeTriangle};
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext as GL;

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

    spawn_local(async {
        let image_element = load_image("horse_shoe.jpg").await.unwrap();
        let gl = context().unwrap();
        let triangle_program = ThreeTriangle::new(&gl);
        gl.use_program(Some(&triangle_program.program));
        let mut angle = 45.0;
        let animation_loop = Rc::new(RefCell::new(None));
        let animation_loop_cloned = animation_loop.clone();
        *animation_loop_cloned.borrow_mut() = Some(Closure::new(move || {
            angle = angle % 360.;
            // angle += 360. / 20.;
            let gl = context().unwrap();
            // point_program
            //     .assign_position(&gl, [0., 0.5, -0.5, -0.5, 0.5, -0.5], angle)
            //     .unwrap();
            if let Err(err) = triangle_program.run(&gl) {
                web_sys::console::log_1(&format!("Failed with error {:#?}", err).into());
            }
            // Specify the color for clearing <canvas>
            // gl.clear_color(0., 0., 0., 1.);
            // gl.clear(GL::COLOR_BUFFER_BIT);

            if angle > 90.0 {
                request_animation_frame(animation_loop.borrow().as_ref().unwrap());
            }
        }));
        request_animation_frame(animation_loop_cloned.borrow().as_ref().unwrap());
    });

    Ok(())
}
