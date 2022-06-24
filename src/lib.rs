mod image_future;
mod images;
mod rain_drops;
mod rain_effect;
mod textures;

use image_future::ImageFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

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

    // Your code goes here!
    console::log_1(&JsValue::from_str("Hello world!"));

    Ok(())
}

#[wasm_bindgen]
pub async fn load_image(path: String) -> HtmlImageElement {
    let image = ImageFuture::new(path.as_ref()).await.unwrap();
    image
}

/// create canvas element by document
///
/// Example:
/// ```rust
/// let (canvas, ctx) = create_canvas_element(640, 320);
/// ```
///
pub fn create_canvas_element(
    w: u32,
    h: u32,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(w);
    canvas.set_height(h);

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    Ok((canvas, ctx))
}
