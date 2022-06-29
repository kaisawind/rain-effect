use crate::rain_drops::{RainDrops, RainDropsOptions};
use crate::rain_render::{RainRender, RainRenderOptions};
use crate::textures::Textures;
use crate::{now, request_animation_frame};
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
    textures: Option<Textures>,
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
        let textures = Textures::new(values).await;

        let mut opts = RainDropsOptions::new();
        opts.trail_rate = 1.0;
        opts.trail_scale_range = [0.2, 0.45];
        opts.collision_radius = 0.45;
        opts.droplets_cleaning_radius_multiplier = 0.28;
        let (w, h) = (canvas.width() as f64, canvas.height() as f64);

        let mut rain_drops = RainDrops::new(
            w * dpi,
            h * dpi,
            dpi,
            Rc::clone(&textures.images.drop),
            Some(opts),
        );
        rain_drops.render_droplets().unwrap();

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
            Rc::clone(&textures.fg),
            Rc::clone(&textures.bg),
            Some(opts),
        );

        let rain_render = Rc::new(RefCell::new(rain_render));

        RainEffect {
            dpi,
            canvas,
            textures: Some(textures),
            rain_drops,
            rain_render,
        }
    }

    pub fn draw(self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let rain_drops = self.rain_drops.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            rain_drops.borrow_mut().update();
            console::log_1(&JsValue::from(now()));
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}
