use std::{cell::RefCell, rc::Rc};

use browser::{button, canvas, context, height, spawn_local, width, window};
use programs::ludo::ludo_program::LudoProgram;
use wasm_bindgen::prelude::*;

mod browser;
mod engine;
mod programs;
mod server;
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

    let dice_button = button("roll-dice").unwrap();

    let gl = context().unwrap();
    let ludo_program = LudoProgram::new(&gl);

    gl.use_program(Some(&ludo_program.program));
    ludo_program
        .render(&gl, crate::programs::ludo::color::Color::Yellow, 45.)
        .unwrap();

    let mut angle = 45.;
    let animation_loop = Rc::new(RefCell::new(None));
    let animation_loop_cloned = animation_loop.clone();
    *animation_loop_cloned.borrow_mut() = Some(Closure::new(move || {
        angle = angle % 360.;
        let gl = context().unwrap();
        if let Err(err) =
            ludo_program.render(&gl, crate::programs::ludo::color::Color::Yellow, angle)
        {
            web_sys::console::log_1(&format!("Failed with error {:#?}", err).into());
        }

        angle += 0.5;
        if angle < 90. {
            request_animation_frame(animation_loop.borrow().as_ref().unwrap());
        } else {
            angle = 45.;
            ludo_program
                .render(&gl, crate::programs::ludo::color::Color::Yellow, angle)
                .unwrap();
        }
    }));
    let button_down_cb = Closure::<dyn FnMut()>::new(move || {
        request_animation_frame(animation_loop_cloned.borrow().as_ref().unwrap());
    });
    dice_button
        .add_event_listener_with_callback("mousedown", button_down_cb.as_ref().unchecked_ref())
        .unwrap();
    button_down_cb.forget();

    Ok(())
}

fn set_canvas_size() -> anyhow::Result<u32> {
    let canvas = canvas().unwrap();

    let height = height()?;
    let widht = width()?;
    let size = widht.min(height);
    canvas.set_height(size);
    canvas.set_width(size);

    Ok(size)
}
