mod drop;
mod image_future;
mod images;
mod rain_drops;
mod rain_effect;
mod rain_render;
mod shader;
mod textures;
mod weather;
mod webgl;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console, window, CanvasRenderingContext2d, Document, HtmlCanvasElement, Performance,
    WebGlProgram, WebGlRenderingContext, WebGlShader,
};

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

/// request animation frame
///
/// Example:
/// ```rust
/// let f = Rc::new(RefCell::new(None));
/// let g = f.clone();
///
/// let mut i = 0;
/// *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
///     if i > 300 {
///         body().set_text_content(Some("All done!"));
///
///         // Drop our handle to this closure so that it will get cleaned
///         // up once we return.
///         let _ = f.borrow_mut().take();
///         return;
///     }
///
///     // Set the body's text content to how many times this
///     // requestAnimationFrame callback has fired.
///     i += 1;
///     let text = format!("requestAnimationFrame has been called {} times.", i);
///     body().set_text_content(Some(&text));
///
///     // Schedule ourself for another requestAnimationFrame callback.
///     request_animation_frame(f.borrow().as_ref().unwrap());
/// }) as Box<dyn FnMut()>));
///
/// request_animation_frame(g.borrow().as_ref().unwrap());
/// ```
///
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

/// ??????????????????????????????????????????????????????
pub fn performance() -> Performance {
    window().unwrap().performance().unwrap()
}

/// ??????????????????????????????????????????????????????????????????????????????
pub fn now() -> f64 {
    performance().now()
}

/// ????????????????????????????????????
///
/// Example:
/// ```rust
/// let start = system_time(performance().timing().request_start());
/// let end = system_time(performance().timing().response_end());
/// ```
///
pub fn system_time(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = ((amt as u32) % 1_000) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

/// ????????????????????????
pub fn document() -> Document {
    window().unwrap().document().unwrap()
}

/// WebGl compile shader
///
/// Example:
/// ```rust
/// let vert_shader = compile_shader(
///     &context,
///     WebGlRenderingContext::VERTEX_SHADER,
///     r#"
///     attribute vec4 position;
///     void main() {
///         gl_Position = position;
///     }
/// "#,
/// ).unwrap();
///
/// let frag_shader = compile_shader(
///     &context,
///     WebGlRenderingContext::FRAGMENT_SHADER,
///     r#"
///     void main() {
///         gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
///     }
/// "#,
/// ).unwrap();
/// ```
///
pub fn compile_shader(
    ctx: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    // ???????????????
    let shader = ctx
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    // ?????????????????????
    ctx.shader_source(&shader, source);
    // ???????????????
    ctx.compile_shader(&shader);

    // ?????????????????????
    if ctx
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        let err = ctx
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader"));
        // ????????????????????????
        ctx.delete_shader(Some(&shader));
        Err(err)
    }
}

/// ????????????
///
/// Example:
/// ```rust
/// let program = link_program(&context, &vert_shader, &frag_shader)?;
/// context.use_program(Some(&program));
/// ```
///
pub fn link_program(
    ctx: &WebGlRenderingContext,
    vert: &WebGlShader,
    frag: &WebGlShader,
) -> Result<WebGlProgram, String> {
    // ????????????
    let program = ctx
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    // ???????????????
    ctx.attach_shader(&program, vert);
    ctx.attach_shader(&program, frag);

    // ????????????
    ctx.link_program(&program);

    // ??????????????????
    if ctx
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        let err = ctx
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object"));
        // ?????????????????????
        ctx.delete_program(Some(&program));
        Err(err)
    }
}
