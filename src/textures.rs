use crate::create_canvas_element;
use crate::images::{Images, Weather};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement};

pub enum BgSize {
    Width = 384,
    Height = 256,
}

pub enum FgSize {
    Width = 96,
    Height = 64,
}

pub struct Texture {
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
}

pub struct Textures {
    fg: Texture,
    bg: Texture,
    images: Images,
}

impl Textures {
    pub async fn new(map: &js_sys::Map) -> Self {
        let alpha = 1.0;
        let (fg, fg_ctx) =
            create_canvas_element(FgSize::Width as u32, FgSize::Height as u32).unwrap();
        fg_ctx.set_global_alpha(alpha);

        let (bg, bg_ctx) =
            create_canvas_element(BgSize::Width as u32, BgSize::Height as u32).unwrap();
        bg_ctx.set_global_alpha(alpha);

        let textures = Textures {
            images: Images::new(map).await,
            fg: Texture {
                canvas: fg,
                ctx: fg_ctx,
            },
            bg: Texture {
                canvas: bg,
                ctx: bg_ctx,
            },
        };

        let weather = textures.images.weather.as_ref().unwrap();
        let (image_fg, image_bg) = match weather {
            Weather::Rain(image)
            | Weather::Fallout(image)
            | Weather::Storm(image)
            | Weather::Sun(image)
            | Weather::Drizzle(image) => (image.fg.as_ref().unwrap(), image.bg.as_ref().unwrap()),
        };
        textures
            .fg
            .ctx
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_fg,
                0.0,
                0.0,
                FgSize::Width as u32 as f64,
                FgSize::Height as u32 as f64,
            )
            .unwrap();
        textures
            .bg
            .ctx
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_bg,
                0.0,
                0.0,
                FgSize::Width as u32 as f64,
                FgSize::Height as u32 as f64,
            )
            .unwrap();

        textures
    }
}
