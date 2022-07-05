use crate::images::{Images, WeatherImage};
use crate::rain_drops::{RainDrops, RainDropsOptions};
use crate::rain_render::{RainRender, RainRenderOptions};
use crate::textures::{BgSize, FgSize, Texture};
use crate::weather::Weather;
use crate::{create_canvas_element, now, request_animation_frame};
use js_sys::Map;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlCanvasElement};

#[wasm_bindgen]
pub struct RainEffect {
    dpi: f64,
    canvas: Rc<RefCell<HtmlCanvasElement>>,
    fg: Rc<RefCell<Texture>>,
    bg: Rc<RefCell<Texture>>,
    weather_data: Rc<RefCell<Weather>>,
    rain_drops: Rc<RefCell<RainDrops>>,
    rain_render: Rc<RefCell<RainRender>>,
}

#[wasm_bindgen]
impl RainEffect {
    #[wasm_bindgen(constructor)]
    pub async fn new(id: String, map: Map) -> Self {
        if id.is_empty() {
            panic!("canvas id is empty!")
        }

        let window = window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(&id).unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let dpi = window.device_pixel_ratio();
        let (w, h) = (
            window.inner_width().unwrap().as_f64().unwrap(),
            window.inner_height().unwrap().as_f64().unwrap(),
        );
        canvas.set_width((w * dpi) as u32);
        canvas.set_height((h * dpi) as u32);
        canvas
            .style()
            .set_property("width", &format!("{}px", w))
            .unwrap();
        canvas
            .style()
            .set_property("height", &format!("{}px", h))
            .unwrap();

        let values: HashMap<String, String> = map.into_serde().unwrap();
        let images = Images::new(values).await;
        let (fg, bg) = RainEffect::create_textures(images.weather.clone());
        let (fg, bg) = (Rc::new(RefCell::new(fg)), Rc::new(RefCell::new(bg)));

        let weather_data = Weather::new_with_img(images.weather.clone());

        let mut opts = RainDropsOptions::new();
        opts.trail_rate = 1.0;
        opts.trail_scale_range = [0.2, 0.45];
        opts.collision_radius = 0.45;
        opts.droplets_cleaning_radius_multiplier = 0.28;
        let (w, h) = (canvas.width() as f64, canvas.height() as f64);

        let mut rain_drops =
            RainDrops::new(w * dpi, h * dpi, dpi, Rc::clone(&images.drop), Some(opts));
        rain_drops.render_droplets().unwrap();
        rain_drops.set_options(weather_data.options());

        let drops_texture = rain_drops.texture.clone();

        let rain_drops = Rc::new(RefCell::new(rain_drops));

        let canvas = Rc::new(RefCell::new(canvas));

        let mut opts = RainRenderOptions::new();
        opts.brightness = 1.04;
        opts.alpha_multiply = 6.0;
        opts.alpha_subtract = 3.0;

        let rain_render = RainRender::new(
            Rc::clone(&canvas),
            drops_texture,
            fg.clone(),
            bg.clone(),
            Some(opts),
        );

        let rain_render = Rc::new(RefCell::new(rain_render));
        let weather_data = Rc::new(RefCell::new(weather_data));

        RainEffect {
            dpi,
            canvas,
            fg,
            bg,
            rain_drops,
            rain_render,
            weather_data,
        }
    }

    fn create_textures(weather: Rc<RefCell<WeatherImage>>) -> (Texture, Texture) {
        let weather = weather.borrow();
        let (image_fg, image_bg) = match &*weather {
            WeatherImage::Rain(image)
            | WeatherImage::Fallout(image)
            | WeatherImage::Storm(image)
            | WeatherImage::Sun(image)
            | WeatherImage::Drizzle(image) => (&image.fg, &image.bg),
        };
        let alpha = 1.0;
        let (fg, fg_ctx) =
            create_canvas_element(FgSize::Width as u32, FgSize::Height as u32).unwrap();
        fg_ctx.set_global_alpha(alpha);

        fg_ctx
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_fg,
                0.0,
                0.0,
                FgSize::Width as u32 as f64,
                FgSize::Height as u32 as f64,
            )
            .unwrap();

        let (bg, bg_ctx) =
            create_canvas_element(BgSize::Width as u32, BgSize::Height as u32).unwrap();
        bg_ctx.set_global_alpha(alpha);

        bg_ctx
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_bg,
                0.0,
                0.0,
                FgSize::Width as u32 as f64,
                FgSize::Height as u32 as f64,
            )
            .unwrap();

        (
            Texture {
                canvas: fg,
                ctx: fg_ctx,
            },
            Texture {
                canvas: bg,
                ctx: bg_ctx,
            },
        )
    }

    pub fn draw(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let rain_drops = self.rain_drops.clone();
        let rain_render = self.rain_render.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            rain_drops.borrow_mut().draw();
            rain_render.borrow().draw();
            console::log_1(&JsValue::from(now()));
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}
