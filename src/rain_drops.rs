use crate::create_canvas_element;
use crate::images::ColorImage;
use crate::textures::Texture;
use std::f64::consts::PI;
use std::time::SystemTime;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement};

const DROP_SIZE: u32 = 64;

pub struct Options {
    time_scale: f64,
    raining: bool,
    droplets_rate: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            time_scale: 1.0,
            raining: true,
            droplets_rate: 50.0,
        }
    }
}

pub struct RainDrops {
    // 全局配置项
    opts: Options,
    // 背景宽度
    width: f64,
    // 背景高度
    height: f64,
    // 缩放比例
    scale: f64,
    // 背景纹理
    texture: Texture,
    // 上一次描画时间
    last_time: SystemTime,
    // 纹理清洁迭代
    cleaning_iterations: f64,

    // 雨滴纹理
    droplets: Texture,
    // 雨滴像素密度
    droplets_pixel_density: f64,
    // 雨滴个数
    droplets_counter: u32,
    // 雨滴纹理图片
    color_image: ColorImage,

    // 雨滴下落画布
    drops_gfx: Vec<HtmlCanvasElement>,
    // 雨滴清理画布
    clear_gfx: Option<HtmlCanvasElement>,
}

impl RainDrops {
    pub fn new(w: f64, h: f64, scale: f64, opts: Option<Options>) -> Self {
        let opts = match opts {
            Some(x) => x,
            None => Options::default(),
        };
        let droplets_pixel_density = 1.0;

        let (canvas, ctx) = create_canvas_element(w as u32, h as u32).unwrap();
        let (droplets, droplets_ctx) = create_canvas_element(
            (w * droplets_pixel_density) as u32,
            (h * droplets_pixel_density) as u32,
        )
        .unwrap();

        RainDrops {
            opts,
            scale,
            droplets_counter: 0,
            cleaning_iterations: 0.0,
            last_time: SystemTime::now(),
            width: w,
            height: h,
            droplets_pixel_density,
            texture: Texture { canvas, ctx },
            droplets: Texture {
                canvas: droplets,
                ctx: droplets_ctx,
            },
            color_image: ColorImage {
                alpha: None,
                color: None,
            },
            drops_gfx: Vec::new(),
            clear_gfx: None,
        }
    }

    pub fn render_droplets(&mut self) -> Result<(), JsValue> {
        let (buf, buf_ctx) = create_canvas_element(DROP_SIZE, DROP_SIZE)?;

        let values = (0..=255).collect::<Vec<_>>();
        self.drops_gfx = values
            .iter()
            .map(|i| {
                buf_ctx.clear_rect(0.0, 0.0, DROP_SIZE as f64, DROP_SIZE as f64);

                // 颜色
                buf_ctx
                    .set_global_composite_operation("source-over")
                    .unwrap();
                buf_ctx
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        self.color_image.color.as_ref().unwrap(),
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();

                // blue overlay, for depth
                buf_ctx.set_global_composite_operation("screen").unwrap();
                buf_ctx.set_fill_style(&JsValue::from(format!("rgba(0,0,{},1)", i)));
                buf_ctx.fill_rect(0.0, 0.0, DROP_SIZE as f64, DROP_SIZE as f64);

                let (drop, drop_ctx) = create_canvas_element(DROP_SIZE, DROP_SIZE).unwrap();
                // alpha
                drop_ctx
                    .set_global_composite_operation("source-over")
                    .unwrap();
                drop_ctx
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        self.color_image.alpha.as_ref().unwrap(),
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();

                // color
                drop_ctx
                    .set_global_composite_operation("source-in")
                    .unwrap();
                drop_ctx
                    .draw_image_with_html_canvas_element_and_dw_and_dh(
                        &buf,
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();
                drop
            })
            .collect::<Vec<HtmlCanvasElement>>();

        let (clear, clear_ctx) = create_canvas_element(DROP_SIZE * 2, DROP_SIZE * 2)?;
        clear_ctx.set_fill_style(&JsValue::from("#000"));
        clear_ctx.begin_path();
        clear_ctx.arc(
            DROP_SIZE as f64,
            DROP_SIZE as f64,
            DROP_SIZE as f64,
            0.0,
            PI * 2.0,
        )?;
        clear_ctx.fill();

        self.clear_gfx = Some(clear);
        Ok(())
    }
    pub fn clear_canvas(&self) {
        self.texture
            .ctx
            .clear_rect(0.0, 0.0, self.width, self.height);
    }

    fn clear_texture(&mut self) {
        self.cleaning_iterations = 50.0;
    }

    pub fn update(&mut self) {
        // clear old texture
        self.clear_canvas();

        let now = SystemTime::now();

        let delta = now.duration_since(self.last_time).unwrap().as_millis() as f64;
        let mut time_scale = delta * 60.0 / 1000.0;
        if time_scale > 1.1 {
            time_scale = 1.1;
        }
        time_scale *= self.opts.time_scale;
        self.last_time = now;
    }

    // 更新雨滴
    fn update_droplets(&mut self, time_scan: f64) {
        if self.cleaning_iterations > 0.0 {
            self.cleaning_iterations -= time_scan;
            self.droplets
                .ctx
                .set_global_composite_operation("destination-out")
                .unwrap();
            self.droplets
                .ctx
                .set_fill_style(&JsValue::from(format!("rgba(0,0,0,{})", 0.05 * time_scan)));
            self.droplets.ctx.fill_rect(
                0.0,
                0.0,
                self.width * self.droplets_pixel_density,
                self.height * self.droplets_pixel_density,
            );
        }

        if self.opts.raining {
            self.droplets_counter +=
                (self.opts.droplets_rate * time_scan * self.area_multiplier()) as u32;
            for _x in [0..=self.droplets_counter].iter() {
                self.droplets_counter -= 1;
            }
        }
    }

    // 更新雨滴下落过程
    fn update_drops(&mut self, time_scan: f64) {}

    fn area(&self) -> f64 {
        let mut scale = self.scale;
        if self.scale == 0.0 {
            scale = 1.0; // 默认不进行缩放
        }
        self.width * self.height / scale
    }

    fn area_multiplier(&self) -> f64 {
        // 当前面积相对XGA分辨率的乘数
        (self.area() / (1024.0 * 768.0)).sqrt()
    }
}
