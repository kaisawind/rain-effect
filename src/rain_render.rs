use crate::compile_shader;
use crate::shader::{FRAGMENT_SHADER, VERTEX_SHADER};
use crate::textures::Texture;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

pub struct RainRenderOptions {
    pub render_shadow: bool,
    pub min_refraction: f64,
    pub max_refraction: f64,
    pub brightness: f64,
    pub alpha_multiply: f64,
    pub alpha_subtract: f64,
    pub parallax_bg: f64,
    pub parallax_fg: f64,
}

impl Default for RainRenderOptions {
    fn default() -> Self {
        RainRenderOptions {
            render_shadow: false,
            min_refraction: 256.0,
            max_refraction: 512.0,
            brightness: 1.0,
            alpha_multiply: 20.0,
            alpha_subtract: 5.0,
            parallax_bg: 5.0,
            parallax_fg: 20.0,
        }
    }
}

impl RainRenderOptions {
    pub fn new() -> Self {
        RainRenderOptions::default()
    }
}

pub struct RainRender {
    // 背景宽度
    width: f64,
    // 背景高度
    height: f64,
    effect_canvas: Rc<RefCell<HtmlCanvasElement>>,
    drops_texture: Rc<RefCell<Texture>>,
    fg: Rc<Texture>,
    bg: Rc<Texture>,
    opts: RainRenderOptions,
}

impl RainRender {
    pub fn new(
        effect_canvas: Rc<RefCell<HtmlCanvasElement>>,
        drops_texture: Rc<RefCell<Texture>>,
        fg: Rc<Texture>,
        bg: Rc<Texture>,
        opts: Option<RainRenderOptions>,
    ) -> Self {
        let opts = match opts {
            Some(opts) => opts,
            None => RainRenderOptions::new(),
        };
        let canvas = effect_canvas.borrow_mut();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        let webgl = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        let vert_shader =
            compile_shader(&webgl, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER).unwrap();

        let vert_shader = compile_shader(
            &webgl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER,
        )
        .unwrap();

        RainRender {
            width,
            height,
            effect_canvas: effect_canvas.clone(),
            drops_texture,
            fg,
            bg,
            opts,
        }
    }
}
