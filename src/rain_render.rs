use crate::shader::{FRAGMENT_SHADER, VERTEX_SHADER};
use crate::textures::Texture;
use crate::webgl::{UniformType, WebGl};
use crate::{compile_shader, create_canvas_element};
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
    shine: Rc<RefCell<Texture>>,
    fg: Rc<RefCell<Texture>>,
    bg: Rc<RefCell<Texture>>,
    opts: RainRenderOptions,
    gl: WebGl,
    parallax_x: f64,
    parallax_y: f64,
}

impl RainRender {
    pub fn new(
        effect_canvas: Rc<RefCell<HtmlCanvasElement>>,
        drops_texture: Rc<RefCell<Texture>>,
        fg: Rc<RefCell<Texture>>,
        bg: Rc<RefCell<Texture>>,
        opts: Option<RainRenderOptions>,
    ) -> Self {
        let opts = match opts {
            Some(opts) => opts,
            None => RainRenderOptions::new(),
        };
        let canvas = effect_canvas.borrow();
        let (w, h) = (canvas.width() as f64, canvas.height() as f64);

        let gl = WebGl::new(effect_canvas.clone(), None);
        let (bg_w, bg_h) = (
            bg.borrow().canvas.width() as f64,
            bg.borrow().canvas.height() as f64,
        );

        gl.create_uniform(UniformType::F2(w as f32, h as f32), "resolution");
        gl.create_uniform(UniformType::F1((bg_w / bg_h) as f32), "textureRatio");
        gl.create_uniform(UniformType::I1(0), "renderShine");
        gl.create_uniform(UniformType::I1(opts.render_shadow as i32), "renderShadow");
        gl.create_uniform(UniformType::F1(opts.min_refraction as f32), "minRefraction");
        gl.create_uniform(
            UniformType::F1((opts.max_refraction - opts.min_refraction) as f32),
            "refractionDelta",
        );
        gl.create_uniform(UniformType::F1(opts.brightness as f32), "brightness");
        gl.create_uniform(UniformType::F1(opts.alpha_multiply as f32), "alphaMultiply");
        gl.create_uniform(UniformType::F1(opts.alpha_subtract as f32), "alphaSubtract");
        gl.create_uniform(UniformType::F1(opts.parallax_bg as f32), "parallaxBg");
        gl.create_uniform(UniformType::F1(opts.parallax_fg as f32), "parallaxFg");

        gl.create_texture(None, 0);

        let (shine, ctx) = create_canvas_element(2, 2).unwrap();

        gl.create_texture(Some(&shine), 1);
        gl.create_uniform(UniformType::I1(1), "textureShine");

        gl.create_texture(Some(&bg.borrow().canvas), 2);
        gl.create_uniform(UniformType::I1(2), "textureFg");

        gl.create_texture(Some(&fg.borrow().canvas), 3);
        gl.create_uniform(UniformType::I1(3), "textureBg");

        RainRender {
            width: w,
            height: h,
            effect_canvas: effect_canvas.clone(),
            drops_texture,
            shine: Rc::new(RefCell::new(Texture { canvas: shine, ctx })),
            fg,
            bg,
            opts,
            gl,
            parallax_x: 0.0,
            parallax_y: 0.0,
        }
    }

    pub fn draw(&self) {
        self.gl.use_program();
        self.gl.create_uniform(
            UniformType::F2(self.parallax_x as f32, self.parallax_y as f32),
            "parallax",
        );

        self.update_texture();
        self.gl.draw();
    }

    pub fn update_textures(&self) {
        self.gl.active_texture(1);
        self.gl.update_texture(&self.shine.borrow().canvas);

        self.gl.active_texture(2);
        self.gl.update_texture(&self.fg.borrow().canvas);

        self.gl.active_texture(3);
        self.gl.update_texture(&self.bg.borrow().canvas);
    }

    pub fn update_texture(&self) {
        self.gl.active_texture(0);
        self.gl.update_texture(&self.drops_texture.borrow().canvas);
    }

    fn setup_weather(&self) {}
}
