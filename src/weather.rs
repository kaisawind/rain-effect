use crate::images::WeatherImage;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub struct WeatherOptions {
    pub raining: bool,
    pub min_r: f64,
    pub max_r: f64,
    pub rain_limit: f64,
    pub rain_chance: f64,
    pub droplets_rate: f64,
    pub droplets_size: [f64; 2],
    pub trail_rate: f64,
    pub trail_scale_range: [f64; 2],
    pub base: Option<Rc<RefCell<WeatherImage>>>,
    pub flash: Option<Rc<RefCell<WeatherImage>>>,
    pub flash_chance: f64,
    pub collision_radius_increase: f64,
}

impl Default for WeatherOptions {
    fn default() -> Self {
        WeatherOptions {
            raining: true,
            min_r: 20.0,
            max_r: 50.0,
            rain_chance: 0.35,
            rain_limit: 6.0,
            droplets_rate: 50.0,
            droplets_size: [3.0, 5.5],
            trail_rate: 1.0,
            trail_scale_range: [0.25, 0.35],
            base: None,
            flash: None,
            flash_chance: 0.0,
            collision_radius_increase: 0.0002,
        }
    }
}

impl WeatherOptions {
    pub fn new() -> Self {
        WeatherOptions::default()
    }
}

pub enum Weather {
    Rain(WeatherOptions),
    Fallout(WeatherOptions),
    Storm(WeatherOptions),
    Sun(WeatherOptions),
    Drizzle(WeatherOptions),
}

impl Weather {
    pub fn new(name: &str, opts: WeatherOptions) -> Self {
        match name {
            "rain" => Weather::Rain(opts),
            "fallout" => Weather::Fallout(opts),
            "storm" => Weather::Storm(opts),
            "sun" => Weather::Sun(opts),
            "drizzle" => Weather::Drizzle(opts),
            _ => Weather::Rain(opts),
        }
    }

    pub fn new_with_img(rc_img: Rc<RefCell<WeatherImage>>) -> Self {
        let mut opts = WeatherOptions::default();
        let img = &*rc_img.borrow();
        match img {
            WeatherImage::Rain(_) => {
                opts.rain_chance = 0.35;
                opts.droplets_rate = 50.0;
                opts.raining = true;
                opts.base = Some(rc_img.clone());

                Weather::Rain(opts)
            }
            WeatherImage::Fallout(_) => {
                opts.min_r = 30.0;
                opts.max_r = 60.0;
                opts.rain_chance = 0.35;
                opts.droplets_rate = 20.0;
                opts.trail_rate = 4.0;
                opts.base = Some(rc_img.clone());
                opts.collision_radius_increase = 0.0;

                Weather::Fallout(opts)
            }
            WeatherImage::Storm(_) => {
                opts.max_r = 55.0;
                opts.rain_chance = 0.4;
                opts.droplets_rate = 80.0;
                opts.droplets_size = [3.0, 5.5];
                opts.trail_rate = 2.5;
                opts.trail_scale_range = [0.25, 0.4];
                opts.base = Some(rc_img.clone());
                opts.flash_chance = 0.1;

                Weather::Storm(opts)
            }
            WeatherImage::Sun(_) => {
                opts.rain_chance = 0.0;
                opts.rain_limit = 0.0;
                opts.droplets_rate = 0.0;
                opts.raining = false;
                opts.base = Some(rc_img.clone());

                Weather::Sun(opts)
            }
            WeatherImage::Drizzle(_) => {
                opts.min_r = 10.0;
                opts.max_r = 40.0;
                opts.rain_chance = 0.15;
                opts.rain_limit = 2.0;
                opts.droplets_rate = 10.0;
                opts.droplets_size = [3.5, 6.0];
                opts.base = Some(rc_img.clone());

                Weather::Drizzle(opts)
            }
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
