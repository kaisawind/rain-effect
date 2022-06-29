use crate::image_future::ImageFuture;
use std::collections::HashMap;
use std::fmt;
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
    pub fn new(name: &str, img: Image) -> Option<Weather> {
        match name {
            "rain" => Some(Weather::Rain(img)),
            "fallout" => Some(Weather::Fallout(img)),
            "storm" => Some(Weather::Storm(img)),
            "sun" => Some(Weather::Sun(img)),
            "drizzle" => Some(Weather::Drizzle(img)),
            _ => None,
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
    pub weather: Option<Weather>,
    pub drop: ColorImage,
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
            weather: Weather::new("rain", img),
            drop: ColorImage {
                alpha: Some(drop_alpha),
                color: Some(drop_color),
            },
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

        self.weather = Weather::new(value, img);
    }
}
