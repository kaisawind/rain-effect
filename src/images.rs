use crate::image_future::ImageFuture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use web_sys::HtmlImageElement;

pub struct Image {
    pub fg: HtmlImageElement,
    pub bg: HtmlImageElement,
}

pub struct ColorImage {
    pub alpha: HtmlImageElement,
    pub color: HtmlImageElement,
}

pub enum WeatherImage {
    Rain(Image),
    Fallout(Image),
    Storm(Image),
    Sun(Image),
    Drizzle(Image),
}

impl WeatherImage {
    pub fn new(name: &str, img: Image) -> WeatherImage {
        match name {
            "rain" => WeatherImage::Rain(img),
            "fallout" => WeatherImage::Fallout(img),
            "storm" => WeatherImage::Storm(img),
            "sun" => WeatherImage::Sun(img),
            "drizzle" => WeatherImage::Drizzle(img),
            _ => WeatherImage::Rain(img),
        }
    }
}

impl fmt::Display for WeatherImage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            WeatherImage::Rain(_) => "rain",
            WeatherImage::Fallout(_) => "fallout",
            WeatherImage::Storm(_) => "storm",
            WeatherImage::Sun(_) => "sun",
            WeatherImage::Drizzle(_) => "drizzle",
        };
        write!(f, "{}", value)
    }
}

pub struct Images {
    pub weather: Rc<RefCell<WeatherImage>>,
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
            fg,
            bg,
        };

        Images {
            values,
            weather: Rc::new(RefCell::new(WeatherImage::new("rain", img))),
            drop: Rc::new(ColorImage {
                alpha: drop_alpha,
                color: drop_color,
            }),
        }
    }

    pub async fn change_weather(&self, value: &str) {
        let fg = value.to_owned() + "Fg";
        let path = self.values.get(&fg).unwrap();
        let fg = ImageFuture::new(path).await.unwrap();

        let bg = value.to_owned() + "Bg";
        let path = self.values.get(&bg).unwrap();
        let bg = ImageFuture::new(path).await.unwrap();

        let img = Image {
            fg,
            bg,
        };

        *self.weather.borrow_mut() = WeatherImage::new(value, img);
    }
}
