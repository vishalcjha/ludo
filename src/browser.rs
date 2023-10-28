#![allow(dead_code)]
use anyhow::{anyhow, Result};
use futures::Future;
use wasm_bindgen::JsCast;

use web_sys::{
    Document, HtmlButtonElement, HtmlCanvasElement, HtmlImageElement, WebGlRenderingContext, Window,
};
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(format!($($t)*).into());
    }
}

pub(crate) use log;
pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("Failed to get window"))
}

pub fn document() -> Result<Document> {
    window()?
        .document()
        .ok_or_else(|| anyhow!("Failed to get document"))
}

pub fn canvas() -> Result<HtmlCanvasElement> {
    document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow!("Failed to get canvas element"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|element| {
            anyhow!(format!(
                "Failed to convert {:#?} into HtmlCanvasElement",
                element
            ))
        })
}

pub fn button(button_id: impl AsRef<str>) -> Result<HtmlButtonElement> {
    document()?
        .get_element_by_id(button_id.as_ref())
        .ok_or_else(|| {
            anyhow!(format!(
                "Failed to get button with id {:?}",
                button_id.as_ref()
            ))
        })?
        .dyn_into::<HtmlButtonElement>()
        .map_err(|element| {
            anyhow!(format!(
                "Failed to convert {:#?} into HtmlCanvasElement",
                element
            ))
        })
}
pub fn context() -> Result<WebGlRenderingContext> {
    canvas()?
        .get_context("webgl")
        .map_err(|js_error| {
            anyhow!(format!(
                "failed to get webgl context : reason {:#?}",
                js_error
            ))
        })?
        .ok_or_else(|| anyhow!("Failed to get webgl context"))?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|context| {
            anyhow!(format!(
                "Failed to get WebGlRenderingContext from {:#?}",
                context
            ))
        })
}

pub fn new_image() -> Result<HtmlImageElement> {
    HtmlImageElement::new()
        .map_err(|js_error| anyhow!(format!("Failed to create image element {:#?}", js_error)))
}

pub fn height() -> Result<u32> {
    let window = window()?;
    let height = window
        .inner_height()
        .map_err(|err| anyhow!(format!("Failed to get height with error {:#?}", err)))?;
    let height = height
        .as_f64()
        .ok_or_else(|| anyhow!("Failed to convert height to number"))?;
    Ok(height as u32)
}

pub fn width() -> Result<u32> {
    let window = window()?;
    let height = window
        .inner_width()
        .map_err(|err| anyhow!(format!("Failed to get height with error {:#?}", err)))?;
    let height = height
        .as_f64()
        .ok_or_else(|| anyhow!("Failed to convert height to number"))?;
    Ok(height as u32)
}
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future)
}
