use crate::textures::Textures;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlCanvasElement};

#[wasm_bindgen]
pub struct RainEffect {
    canvas: HtmlCanvasElement,
}

#[wasm_bindgen]
impl RainEffect {
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str) -> Self {
        if id.is_empty() {
            panic!("canvas id is empty!")
        }
        let document = window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(id).unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        RainEffect { canvas: canvas }
    }

    pub fn draw(&mut self) {}
}
