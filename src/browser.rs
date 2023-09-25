use anyhow::{anyhow, Result};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlCanvasElement, WebGlRenderingContext, Window};
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
