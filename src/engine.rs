use std::{cell::Cell, rc::Rc};

use anyhow::Result;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::HtmlImageElement;

use crate::browser::new_image;
pub async fn load_image(src: impl AsRef<str>) -> Result<HtmlImageElement> {
    let image = new_image()?;

    let (sender, receiver) =
        futures::channel::oneshot::channel::<core::result::Result<(), JsValue>>();
    let success_tx = Rc::new(Cell::new(Some(sender)));
    let error_tx = success_tx.clone();

    let error_callback = Closure::once(move |err| {
        if let Some(tx) = error_tx.to_owned().take() {
            let _ = tx.send(Err(err));
        };
    });

    let success_callback = Closure::once(move || {
        if let Some(tx) = success_tx.to_owned().take() {
            let _ = tx.send(Ok(()));
        }
    });

    image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
    image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    image.set_src(src.as_ref());

    let _ = receiver.await;
    Ok(image)
}
