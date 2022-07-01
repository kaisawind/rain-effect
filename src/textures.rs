use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

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
