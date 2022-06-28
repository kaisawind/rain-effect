use std::cell::RefCell;
use std::rc::Rc;

pub struct Drop {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub spread_x: f64,
    pub spread_y: f64,
    pub momentum: f64,
    pub momentum_x: f64,
    pub last_spawn: f64,
    pub next_spawn: f64,
    pub parent: Option<Rc<RefCell<Drop>>>,
    pub is_new: bool,
    pub killed: bool,
    pub shrink: f64,
}

impl Default for Drop {
    fn default() -> Self {
        Drop {
            x: 0.0,
            y: 0.0,
            r: 0.0,
            spread_x: 0.0,
            spread_y: 0.0,
            momentum: 0.0,
            momentum_x: 0.0,
            last_spawn: 0.0,
            next_spawn: 0.0,
            parent: None,
            is_new: true,
            killed: false,
            shrink: 0.0,
        }
    }
}

impl Drop {
    pub fn new() -> Rc<RefCell<Self>> {
        let drop = Drop::default();

        Rc::new(RefCell::new(drop))
    }
}
