use crate::image_future::ImageFuture;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use web_sys::HtmlImageElement;

pub struct Image {
    pub fg: Option<HtmlImageElement>,
    pub bg: Option<HtmlImageElement>,
}

pub struct ColorImage {
    pub alpha: Option<HtmlImageElement>,
    pub color: Option<HtmlImageElement>,
}

pub enum Weather {
    Rain(Image),
    Fallout(Image),
    Storm(Image),
    Sun(Image),
    Drizzle(Image),
}

impl Weather {
    pub fn new(name: &str, img: Image) -> Weather {
        match name {
            "rain" => Weather::Rain(img),
            "fallout" => Weather::Fallout(img),
            "storm" => Weather::Storm(img),
            "sun" => Weather::Sun(img),
            "drizzle" => Weather::Drizzle(img),
            _ => Weather::Rain(img),
        }
    }
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            Weather::Rain(_) => "rain",
            Weather::Fallout(_) => "fallout",
            Weather::Storm(_) => "storm",
            Weather::Sun(_) => "sun",
            Weather::Drizzle(_) => "drizzle",
        };
        write!(f, "{}", value)
    }
}

pub struct Images {
    pub weather: Rc<Weather>,
    pub drop: Rc<ColorImage>,
    values: HashMap<String, String>,
}

impl Images {
    pub async fn new(values: HashMap<String, String>) -> Self {
        let path = values.get("dropAlpha").unwrap();
        let drop_alpha = ImageFuture::new(path).await.unwrap();

        let path = values.get("dropColor").unwrap();
        let drop_color = ImageFuture::new(path).await.unwrap();

        let fg = "rainFg";
        let path = values.get(fg).unwrap();
        let fg = ImageFuture::new(path).await.unwrap();

        let bg = "rainBg";
        let path = values.get(bg).unwrap();
        let bg = ImageFuture::new(path).await.unwrap();

        let img = Image {
            fg: Some(fg),
            bg: Some(bg),
        };

        Images {
            values,
            weather: Rc::new(Weather::new("rain", img)),
            drop: Rc::new(ColorImage {
                alpha: Some(drop_alpha),
                color: Some(drop_color),
            }),
        }
    }

    pub async fn change_weather(mut self, value: &str) {
        let fg = value.to_owned() + "Fg";
        let path = self.values.get(&fg).unwrap();
        let fg = ImageFuture::new(path).await.unwrap();

        let bg = value.to_owned() + "Bg";
        let path = self.values.get(&bg).unwrap();
        let bg = ImageFuture::new(path).await.unwrap();

        let img = Image {
            fg: Some(fg),
            bg: Some(bg),
        };

        self.weather = Rc::new(Weather::new(value, img));
    }
}
