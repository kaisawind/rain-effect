use crate::shader::{FRAGMENT_SHADER, VERTEX_SHADER};
use crate::{compile_shader, link_program};
use js_sys::Float32Array;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, HtmlCanvasElement, WebGlProgram, WebGlRenderingContext};

#[derive(Serialize, Deserialize, Debug)]
pub struct WebGlOptions {
    pub alpha: bool,
}

impl Default for WebGlOptions {
    fn default() -> Self {
        WebGlOptions { alpha: false }
    }
}

impl WebGlOptions {
    pub fn new() -> Self {
        WebGlOptions::default()
    }
}

pub enum UniformType {
    F1(f32),
    I1(i32),
    F2(f32, f32),
}

pub struct WebGl {
    // 背景宽度
    width: f64,
    // 背景高度
    height: f64,
    canvas: Rc<RefCell<HtmlCanvasElement>>,
    gl: WebGlRenderingContext,
    program: WebGlProgram,
}

impl WebGl {
    pub fn new(canvas: Rc<RefCell<HtmlCanvasElement>>, opts: Option<WebGlOptions>) -> Self {
        let opts = match opts {
            Some(opts) => opts,
            None => WebGlOptions::new(),
        };
        let opts = JsValue::from_serde(&opts).unwrap();
        let gl = canvas
            .borrow()
            .get_context_with_context_options("webgl", &opts)
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        let width = canvas.borrow().width() as f64;
        let height = canvas.borrow().height() as f64;

        let program = WebGl::create_program(&gl);

        gl.use_program(program.as_ref());

        WebGl {
            program: program.unwrap(),
            gl,
            canvas,
            width,
            height,
        }
    }

    fn create_program(context: &WebGlRenderingContext) -> Option<WebGlProgram> {
        let vert_shader =
            compile_shader(context, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER).unwrap();

        let frag_shader = compile_shader(
            context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER,
        )
        .unwrap();

        let program = link_program(&context, &vert_shader, &frag_shader).unwrap();

        let linked = context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS);
        if !linked.as_bool().unwrap() {
            let err = context.get_program_info_log(&program).unwrap();
            console::error_1(&JsValue::from(err));
            context.delete_program(Some(&program));
            return None;
        }

        let a_position = context.get_attrib_location(&program, "a_position");
        let a_tex_coord = context.get_attrib_location(&program, "a_texCoord");

        let buffer = context.create_buffer();
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, buffer.as_ref());
        let vertices: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
        ];
        unsafe {
            let vert_array = Float32Array::view(&vertices);
            context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        context.enable_vertex_attrib_array(a_tex_coord as u32);
        context.vertex_attrib_pointer_with_i32(
            a_tex_coord as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );

        let buffer = context.create_buffer();
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, buffer.as_ref());

        context.enable_vertex_attrib_array(a_position as u32);
        context.vertex_attrib_pointer_with_i32(
            a_position as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );

        Some(program)
    }

    pub fn use_program(&self) {
        self.gl.use_program(Some(&self.program));
    }

    pub fn create_uniform(&self, ut: UniformType, name: &str) {
        let location = self
            .gl
            .get_uniform_location(&self.program, &format!("u{}", name));
        match ut {
            UniformType::F1(x) => {
                self.gl.uniform1f(location.as_ref(), x);
            }
            UniformType::I1(x) => {
                self.gl.uniform1i(location.as_ref(), x);
            }
            UniformType::F2(x, y) => {
                self.gl.uniform2f(location.as_ref(), x, y);
            }
        };
    }

    pub fn create_texture(&self, source: Option<&HtmlCanvasElement>, idx: u32) {
        let texture = self.gl.create_texture();
        self.active_texture(idx);
        self.gl
            .bind_texture(WebGlRenderingContext::TEXTURE_2D, texture.as_ref());

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_S,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_T,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::LINEAR as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::LINEAR as i32,
        );

        match source {
            Some(source) => self.update_texture(source),
            None => {}
        };
    }

    pub fn active_texture(&self, idx: u32) {
        self.gl.active_texture(idx);
    }

    pub fn update_texture(&self, source: &HtmlCanvasElement) {
        self.gl
            .tex_image_2d_with_u32_and_u32_and_canvas(
                WebGlRenderingContext::TEXTURE_2D,
                0,
                WebGlRenderingContext::RGBA as i32,
                WebGlRenderingContext::RGBA,
                WebGlRenderingContext::UNSIGNED_BYTE,
                source,
            )
            .unwrap();
    }

    fn set_rectangle(&self, x: f32, y: f32, w: f32, h: f32) {
        let (x1, x2, y1, y2) = (x, x + w, y, y + h);
        let vertices: [f32; 12] = [x1, y1, x2, y1, x1, y2, x1, y2, x2, y1, x2, y2];
        unsafe {
            let vert_array = Float32Array::view(&vertices);
            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
    }

    pub fn draw(&self) {
        self.set_rectangle(-1.0, -1.0, 2.0, 2.0);
        self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
    }
}
