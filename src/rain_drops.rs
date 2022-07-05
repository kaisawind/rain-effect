use crate::drop::Drop;
use crate::images::ColorImage;
use crate::textures::Texture;
use crate::weather::WeatherOptions;
use crate::{create_canvas_element, now};
use js_sys::Math::{max, min};
use rand::{thread_rng, Rng};
use std::cell::{RefCell, RefMut};
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement};

const DROP_SIZE: u32 = 64;

pub struct RainDropsOptions {
    /// 时标比率（time_scale *= time_scale_multiplier）
    pub time_scale_multiplier: f64,
    pub raining: bool,
    /// XGA(1024 x 768)分辨率下每帧雨滴数量
    pub droplets_rate: f64,

    /// 雨点半径大小
    ///
    /// Example:
    /// ```rust
    /// let (min, max) = r;
    /// ```
    ///
    pub r: (f64, f64),
    pub max_drops: i32,

    /// 雨点大小
    ///
    /// Example:
    /// ```rust
    /// let (min, max) = droplets_size;
    /// ```
    pub droplets_size: (f64, f64),
    pub droplets_cleaning_radius_multiplier: f64,
    pub drop_fall_multiplier: f64,

    /// 雨量限制（一次最多生成多少雨点）
    pub rain_limit: f64,
    /// 雨点生成几率
    pub rain_chance: f64,

    /// 雨点生成区域
    pub spawn_area: [f64; 2],
    pub auto_shrink: bool,
    pub trail_rate: f64,
    pub trail_scale_range: [f64; 2],
    pub global_time_scale: f64,
    pub collision_radius: f64,
    pub collision_radius_increase: f64,
    pub collision_boost_multiplier: f64,
    pub collision_boost: f64,
}

impl Default for RainDropsOptions {
    fn default() -> Self {
        RainDropsOptions {
            r: (10.0, 40.0),
            time_scale_multiplier: 1.0,
            raining: true,
            droplets_rate: 50.0,
            droplets_size: (2.0, 4.0),
            drop_fall_multiplier: 1.0,
            rain_limit: 3.0,
            rain_chance: 0.3,
            spawn_area: [-0.1, 0.95],
            auto_shrink: true,
            trail_rate: 1.0,
            max_drops: 900,
            trail_scale_range: [0.2, 0.5],
            global_time_scale: 1.0,
            collision_radius: 0.65,
            collision_radius_increase: 0.01,
            collision_boost_multiplier: 0.05,
            collision_boost: 1.0,
            droplets_cleaning_radius_multiplier: 0.43,
        }
    }
}

impl RainDropsOptions {
    pub fn new() -> Self {
        RainDropsOptions::default()
    }
}

pub struct RainDrops {
    // 全局配置项
    opts: RainDropsOptions,
    // 背景宽度
    width: f64,
    // 背景高度
    height: f64,
    // 缩放比例
    scale: f64,
    // 背景纹理
    pub texture: Rc<RefCell<Texture>>,
    // 上一次描画时间
    last_time: f64,
    // 纹理清理迭代
    cleaning_iterations: f64,

    // 雨滴纹理
    droplets: Texture,
    // 雨滴像素密度
    droplets_pixel_density: f64,
    // 雨滴个数
    droplets_counter: u32,
    // 雨滴纹理图片
    color_image: Rc<ColorImage>,

    // 雨滴
    drops: Vec<Rc<RefCell<Drop>>>,
    // 雨滴下落画布
    drops_gfx: Vec<HtmlCanvasElement>,
    // 雨滴清理画布
    clear_gfx: Option<HtmlCanvasElement>,
}

impl RainDrops {
    pub fn new(
        w: f64,
        h: f64,
        scale: f64,
        color_image: Rc<ColorImage>,
        opts: Option<RainDropsOptions>,
    ) -> Self {
        // 初期化雨滴参数
        let opts = match opts {
            Some(x) => x,
            None => RainDropsOptions::default(),
        };

        // 水滴像素密度： 默认1像素
        let droplets_pixel_density = 1.0;

        // 创建背景画布
        let (canvas, ctx) = create_canvas_element(w as u32, h as u32).unwrap();
        // 根据像素密度创建雨滴画布
        let (droplets, droplets_ctx) = create_canvas_element(
            (w * droplets_pixel_density) as u32,
            (h * droplets_pixel_density) as u32,
        )
        .unwrap();
        let last_time = now();
        RainDrops {
            opts,
            scale,
            droplets_counter: 0,
            cleaning_iterations: 0.0,
            last_time,
            width: w,
            height: h,
            droplets_pixel_density,
            texture: Rc::new(RefCell::new(Texture { canvas, ctx })),
            droplets: Texture {
                canvas: droplets,
                ctx: droplets_ctx,
            },
            color_image,
            drops: Vec::new(),
            drops_gfx: Vec::new(),
            clear_gfx: None,
        }
    }

    pub fn render_droplets(&mut self) -> Result<(), JsValue> {
        let (buf, buf_ctx) = create_canvas_element(DROP_SIZE, DROP_SIZE)?;

        let values = (0..255).collect::<Vec<_>>();
        self.drops_gfx = values
            .iter()
            .map(|i| {
                let (drop, drop_ctx) = create_canvas_element(DROP_SIZE, DROP_SIZE).unwrap();

                buf_ctx.clear_rect(0.0, 0.0, DROP_SIZE as f64, DROP_SIZE as f64);

                // 颜色
                buf_ctx
                    .set_global_composite_operation("source-over")
                    .unwrap();
                buf_ctx
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        &self.color_image.color,
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();

                // blue overlay, for depth
                buf_ctx.set_global_composite_operation("screen").unwrap();
                buf_ctx.set_fill_style(&JsValue::from(format!("rgba(0,0,{},1)", i)));
                buf_ctx.fill_rect(0.0, 0.0, DROP_SIZE as f64, DROP_SIZE as f64);

                // alpha
                drop_ctx
                    .set_global_composite_operation("source-over")
                    .unwrap();
                drop_ctx
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        &self.color_image.alpha,
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();

                // color
                drop_ctx
                    .set_global_composite_operation("source-in")
                    .unwrap();
                drop_ctx
                    .draw_image_with_html_canvas_element_and_dw_and_dh(
                        &buf,
                        0.0,
                        0.0,
                        DROP_SIZE as f64,
                        DROP_SIZE as f64,
                    )
                    .unwrap();
                drop
            })
            .collect::<Vec<HtmlCanvasElement>>();

        let (clear, clear_ctx) = create_canvas_element(DROP_SIZE * 2, DROP_SIZE * 2)?;
        clear_ctx.set_fill_style(&JsValue::from("#000"));
        clear_ctx.begin_path();
        clear_ctx.arc(
            DROP_SIZE as f64,
            DROP_SIZE as f64,
            DROP_SIZE as f64,
            0.0,
            PI * 2.0,
        )?;
        clear_ctx.fill();

        self.clear_gfx = Some(clear);
        Ok(())
    }

    fn draw_drop(&self, ctx: &CanvasRenderingContext2d, drop: RefMut<Drop>) {
        if !self.drops_gfx.is_empty() {
            let x = drop.x;
            let y = drop.y;
            let r = drop.r;
            let spread_x = drop.spread_x;
            let spread_y = drop.spread_y;
            let (min_r, _max_r) = self.opts.r;

            let scale_x = 1.0;
            let scale_y = 1.5;
            let mut d = max(0.0, min(1.0, ((r - min_r) / self.delta_r()) * 0.9));
            d *= 1.0 / (((spread_x + spread_y) * 0.5) + 1.0);
            let d = (d * (self.drops_gfx.len() - 1) as f64).floor();

            ctx.set_global_alpha(1.0);
            // 新图像会覆盖在原有图像
            ctx.set_global_composite_operation("source-over").unwrap();
            ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                &self.drops_gfx[d as usize],
                (x - (r * scale_x * (spread_x + 1.0))) * self.scale,
                (y - (r * scale_y * (spread_y + 1.0))) * self.scale,
                (r * 2.0 * scale_x * (spread_x + 1.0)) * self.scale,
                (r * 2.0 * scale_y * (spread_y + 1.0)) * self.scale,
            )
            .unwrap();
        }
    }

    fn draw_droplet(&self, x: f64, y: f64, r: f64) {
        let rc_drop = Drop::new();
        let drop = rc_drop.clone();
        drop.borrow_mut().x = x * self.droplets_pixel_density;
        drop.borrow_mut().y = y * self.droplets_pixel_density;
        drop.borrow_mut().r = r * self.droplets_pixel_density;
        self.draw_drop(&self.droplets.ctx, drop.borrow_mut());
    }

    fn clear_droplets(&self, x: f64, y: f64, r: Option<f64>) {
        let r = match r {
            Some(r) => r,
            None => 40.0,
        };
        let multiplier = self.droplets_pixel_density * self.scale;
        self.droplets
            .ctx
            .set_global_composite_operation("destination-out")
            .unwrap();
        self.droplets
            .ctx
            .draw_image_with_html_canvas_element_and_dw_and_dh(
                self.clear_gfx.as_ref().unwrap(),
                (x - r) * multiplier,
                (y - r) * multiplier,
                (r * 2.0) * multiplier,
                (r * 2.0) * multiplier * 1.5,
            )
            .unwrap();
    }

    /// 清理画布
    fn clear_canvas(&self) {
        self.texture
            .borrow_mut()
            .ctx
            .clear_rect(0.0, 0.0, self.width, self.height);
    }

    fn clear_texture(&mut self) {
        self.cleaning_iterations = 50.0;
    }

    pub fn draw(&mut self) {
        // 初期化画布
        self.clear_canvas();

        // 当前计数(毫秒)
        let now = now();
        let delta = now - self.last_time;
        // time_scale = delta时间内运行了帧动画
        // 限制刷新频率 60FPS(60/s)
        // time_scale = delta / ((1 / 60) * 1000)
        let mut time_scale = delta * 60.0 / 1000.0;
        if time_scale > 1.1 {
            time_scale = 1.1;
        }
        time_scale *= self.opts.time_scale_multiplier;
        self.last_time = now;

        self.update_drops(time_scale);
    }

    /// 更新雨滴
    fn update_droplets(&mut self, time_scan: f64) {
        // 渐变消去的效果
        if self.cleaning_iterations > 0.0 {
            self.cleaning_iterations -= time_scan;

            // 绘制原图和新图不重叠部分
            self.droplets
                .ctx
                .set_global_composite_operation("destination-out")
                .unwrap();

            // 半透明黑色
            self.droplets
                .ctx
                .set_fill_style(&JsValue::from(format!("rgba(0,0,0,{})", 0.05 * time_scan)));
            self.droplets.ctx.fill_rect(
                0.0,
                0.0,
                self.width * self.droplets_pixel_density,
                self.height * self.droplets_pixel_density,
            );
        }

        if self.opts.raining {
            // 根据 分辨率+时间尺度+面积系数 计算累积雨滴数量
            self.droplets_counter +=
                (self.opts.droplets_rate * time_scan * self.area_multiplier()) as u32;
            let mut rng = thread_rng();
            let (min, max) = self.opts.droplets_size;
            let (w, h) = (
                (self.width / self.scale) as i32,
                (self.height / self.scale) as i32,
            );
            while self.droplets_counter > 0 {
                let x = rng.gen_range(0..w) as f64;
                let y = rng.gen_range(0..h) as f64;
                // 更多的小雨滴
                let r = min + rng.gen::<f64>().powi(2) * (max - min);
                self.draw_droplet(x, y, r);
                self.droplets_counter -= 1;
            }
        }

        self.texture
            .borrow_mut()
            .ctx
            .draw_image_with_html_canvas_element_and_dw_and_dh(
                &self.droplets.canvas,
                0.0,
                0.0,
                self.width,
                self.height,
            )
            .unwrap();
    }

    fn gen_drops(&self, time_scan: f64) -> Vec<Rc<RefCell<Drop>>> {
        let mut drops: Vec<Rc<RefCell<Drop>>> = Vec::new();
        if self.opts.raining {
            // 雨量限制
            let limit = (self.opts.rain_limit * time_scan * self.area_multiplier()) as i32;
            // 下雨几率
            let chance = self.opts.rain_chance * time_scan * self.area_multiplier();

            let mut count = 0;
            let mut rng = thread_rng();
            let (min, max) = self.opts.r;
            let (w, h) = (self.width / self.scale, self.height / self.scale);
            // 雨点在Y轴生成范围
            let [spawn_min, spawn_max] = self.opts.spawn_area.map(|x| x * h);
            while rng.gen::<f64>() <= chance && count < limit {
                count += 1;
                let x = rng.gen_range(0..w as i32) as f64;
                let y = rng.gen_range(spawn_min as i32..spawn_max as i32) as f64;
                let n = rng.gen::<f64>().powi(3);
                let r = min + n * (max - min);
                let momentum = 1.0 + (r - min) * 0.1 + rng.gen::<f64>() * 2.0;

                if !self.is_full_drops() {
                    let drop = Drop::new();

                    drop.borrow_mut().x = x;
                    drop.borrow_mut().y = y;
                    drop.borrow_mut().r = r;
                    drop.borrow_mut().momentum = momentum;
                    drop.borrow_mut().spread_x = 1.5;
                    drop.borrow_mut().spread_y = 1.5;

                    drops.push(drop.clone());
                }
            }
        }

        drops
    }

    // 更新雨滴下落过程
    fn update_drops(&mut self, time_scan: f64) {
        self.update_droplets(time_scan);
        let mut drops = self.gen_drops(time_scan);

        let (w, h) = (self.width / self.scale, self.height / self.scale);

        self.drops.sort_by(|a, b| {
            let va = (a.borrow().y * w + a.borrow().x) as i32;
            let vb = (b.borrow().y * w + b.borrow().x) as i32;
            va.cmp(&vb)
        });

        let (min_r, max_r) = self.opts.r;
        let mut rng = thread_rng();

        let drop_fall = min_r * self.opts.drop_fall_multiplier;
        let delta_r = 0.1 / self.delta_r() * time_scan;
        let is_full_drops = self.is_full_drops();
        for (i, rc_drop) in self.drops.iter().enumerate() {
            let mut drop = rc_drop.borrow_mut();
            console::log_4(
                &JsValue::from(format!("drop[{}]", i as i32)),
                &JsValue::from(drop.x),
                &JsValue::from(drop.y),
                &JsValue::from(drop.r),
            );
            if !drop.killed {
                // 更新重力
                // 雨滴下滑的几率
                if rng.gen::<f64>() < (drop.r - drop_fall) * delta_r {
                    drop.momentum += rng.gen::<f64>() * (drop.r / max_r * 4.0);
                }

                // 清除小的雨滴
                if self.opts.auto_shrink && drop.r <= min_r && rng.gen::<f64>() < 0.05 * time_scan {
                    drop.shrink += 0.01;
                }

                // 收缩雨滴
                drop.r -= drop.shrink * time_scan;
                if drop.r <= 0.0 {
                    drop.killed = true;
                }

                // 更新雨迹
                if self.opts.raining {
                    drop.last_spawn += drop.momentum * time_scan * self.opts.trail_rate;
                    if drop.last_spawn > drop.next_spawn {
                        if !is_full_drops {
                            let new_drop = Drop::new();
                            new_drop.borrow_mut().x =
                                drop.x + (-drop.r + rng.gen::<f64>() * 2.0 * drop.r) * 0.1;
                            new_drop.borrow_mut().y = drop.y - drop.r * 0.01;
                            let [trail_min, trail_max] = self.opts.trail_scale_range;
                            new_drop.borrow_mut().r =
                                drop.r * (trail_min + rng.gen::<f64>() * (trail_max - trail_min));
                            new_drop.borrow_mut().spread_y = drop.momentum * 0.1;
                            new_drop.borrow_mut().parent = Some(Rc::clone(rc_drop));

                            drops.push(Rc::clone(&new_drop));

                            drop.r *= 0.97_f32.powf(time_scan as f32) as f64;
                            drop.last_spawn = 0.0;
                            drop.next_spawn = min_r + rng.gen::<f64>() * (max_r - min_r)
                                - (drop.momentum * 2.0 * self.opts.trail_rate)
                                + (max_r - drop.r);
                        }
                    }
                }

                // 标准流动
                drop.spread_x *= 0.4_f32.powf(time_scan as f32) as f64;
                drop.spread_y *= 0.7_f32.powf(time_scan as f32) as f64;

                // 更新位置
                let moved = drop.momentum > 0.0;
                if moved && !drop.killed {
                    drop.y += drop.momentum * self.opts.global_time_scale;
                    drop.x += drop.momentum_x * self.opts.global_time_scale;
                    if drop.y > h + drop.r {
                        drop.killed = true;
                    }
                }

                // 碰撞
                let collision = (moved || drop.is_new) && !drop.killed;
                drop.is_new = false;

                if collision {
                    let mut slice: Vec<Rc<RefCell<Drop>>> = Vec::new();
                    if i + 70 <= self.drops.len() {
                        slice = self.drops[i + 1..i + 70].to_vec();
                    } else if i + 1 < self.drops.len() && i + 70 > self.drops.len() {
                        slice = self.drops[i + 1..].to_vec();
                    }
                    for rc_drop2 in slice.iter() {
                        let mut drop2 = rc_drop2.borrow_mut();
                        if !Rc::ptr_eq(rc_drop, rc_drop2) && drop.r > drop2.r && !drop2.killed {
                            // 比较父雨点是否相同
                            let parent_eq = match (&drop.parent, &drop2.parent) {
                                (Some(drop), Some(drop2)) => Rc::ptr_eq(drop, drop2),
                                (None, None) => true,
                                _ => false,
                            };
                            if !parent_eq {
                                let dx = drop2.x - drop.x;
                                let dy = drop2.y - drop.y;
                                let d = (dx.powi(2) + dy.powi(2)).sqrt();
                                if d < (drop.r + drop2.r)
                                    * (self.opts.collision_radius
                                        + drop.momentum
                                            * self.opts.collision_radius_increase
                                            * time_scan)
                                {
                                    let a1 = PI * drop.r.powi(2); // drop 的面积
                                    let a2 = PI * drop2.r.powi(2); // drop2 的面积
                                    let mut r = ((a1 + a2 * 0.8) / PI).sqrt(); // 两个雨点合并之后半径
                                    if r > max_r {
                                        r = max_r;
                                    }
                                    drop.r = r;
                                    drop.momentum_x += dx * 0.1;
                                    drop.spread_x = 0.0;
                                    drop.spread_y = 0.0;
                                    drop.momentum = max(
                                        drop2.momentum,
                                        min(
                                            40.0,
                                            drop.momentum
                                                + r * self.opts.collision_boost_multiplier
                                                + self.opts.collision_boost,
                                        ),
                                    );
                                    drop2.killed = true;
                                }
                            }
                        }
                    }
                }

                // 放慢流动速度
                drop.momentum -= max(1.0, min_r * 0.5 - drop.momentum) * 0.1 * time_scan;
                if drop.momentum < 0.0 {
                    drop.momentum = 0.0;
                }

                drop.momentum_x *= 0.7_f32.powf(time_scan as f32) as f64;

                if !drop.killed {
                    drops.push(Rc::clone(&rc_drop));
                    if moved && self.opts.droplets_rate > 0.0 {
                        let r = drop.r * self.opts.droplets_cleaning_radius_multiplier;
                        self.clear_droplets(drop.x, drop.y, Some(r));
                    }

                    self.draw_drop(&self.texture.borrow_mut().ctx, drop);
                }
            }
        }

        self.drops = drops;
    }

    fn is_full_drops(&self) -> bool {
        (self.drops.len() as i32) >= (self.opts.max_drops as f64 * self.area_multiplier()) as i32
    }

    /// 雨滴区域的面积
    fn area(&self) -> f64 {
        let mut scale = self.scale;
        if self.scale == 0.0 {
            scale = 1.0; // 默认不进行缩放
        }
        self.width * self.height / scale
    }

    /// 当前面积相对XGA分辨率的乘数
    fn area_multiplier(&self) -> f64 {
        (self.area() / (1024.0 * 768.0)).sqrt()
    }

    /// 雨滴半径差
    fn delta_r(&self) -> f64 {
        let (min, max) = self.opts.r;
        max - min
    }

    pub fn set_options(&mut self, opts: &WeatherOptions) {
        self.opts.raining = opts.raining;
        self.opts.r = opts.r;
        self.opts.rain_chance = opts.rain_chance;
        self.opts.rain_limit = opts.rain_limit;
        self.opts.droplets_rate = opts.droplets_rate;
        self.opts.droplets_size = opts.droplets_size;
        self.opts.trail_rate = opts.trail_rate;
        self.opts.trail_scale_range = opts.trail_scale_range;
        self.opts.collision_radius_increase = opts.collision_radius_increase;
    }
}
