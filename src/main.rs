use std::error::Error;

use instant::{Duration, Instant};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

mod scene;
use macroquad::rand::gen_range;
use scene::*;

#[cfg(not(target_arch = "wasm32"))]
const FPS: u32 = 144;

const FONT_START: f32 = 30.0;
const FONT_FIN: f32 = 30.0;
const FONT_SCORE: f32 = 18.0;

#[derive(Serialize, Deserialize)]
pub struct Config {
    size: f32,
    wait_time: u64,
    target_time: u64,
}

pub struct Data {
    pub score: u32,
    pub count: u32,
    pub was_pressed: bool,
    pub width: f32,
    pub height: f32,
    pub timer_start: Instant,
    pub config: Config,
}

fn get_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| include_str!("../assets/default_config.toml").to_string());
    Ok(toml::from_str(&contents)?)
}

impl Default for Data {
    fn default() -> Self {
        Self {
            score: Default::default(),
            count: Default::default(),
            was_pressed: Default::default(),
            width: Default::default(),
            height: Default::default(),
            timer_start: Instant::now(),
            config: get_config("config.toml").expect("Could not parse toml!"),
        }
    }
}

pub struct Start;

impl Scene<Data, ()> for Start {
    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        if macroquad::input::is_mouse_button_pressed(MouseButton::Left) {
            SceneSwitch::replace(Waiting {
                until: Duration::from_millis(world.config.wait_time),
                background: WHITE,
            })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data) {
        draw_text(
            "Click to start!",
            world.width / 2.0,
            world.height / 2.0,
            FONT_START,
            BLACK,
        );
    }
}

pub struct Waiting {
    pub until: Duration,
    pub background: Color,
}

fn draw_score(world: &mut Data) {
    draw_text(
        &format!("{} / {}", world.score, world.count),
        10.0,
        5.0 + FONT_SCORE,
        FONT_SCORE,
        BLACK,
    );
}

impl Scene<Data, ()> for Waiting {
    fn draw(&mut self, world: &mut Data) {
        clear_background(self.background);
        draw_score(world);
    }

    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        if world.count > 50 {
            return SceneSwitch::replace(Fin);
        }
        if Instant::now().duration_since(world.timer_start) > self.until {
            let v = gen_range(0, 360u32) as f32;
            world.timer_start = Instant::now();
            let size = world.config.size / 2.0;
            SceneSwitch::replace(Target {
                until: Duration::from_millis(world.config.target_time),
                min: (v - size).rem_euclid(360.0),
                max: (v + size).rem_euclid(360.0),
            })
        } else {
            SceneSwitch::None
        }
    }
}

pub struct Target {
    pub until: Duration,
    pub min: f32,
    pub max: f32,
}

impl Scene<Data, ()> for Target {
    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        if world.count > 50 {
            return SceneSwitch::replace(Fin);
        }
        if Instant::now().duration_since(world.timer_start) > self.until {
            world.count += 1;
            world.timer_start = Instant::now();
            SceneSwitch::replace(Waiting {
                until: Duration::from_millis(world.config.wait_time),
                background: RED,
            })
        } else {
            let is_pressed = is_mouse_button_pressed(MouseButton::Left);
            if is_pressed && world.was_pressed == false {
                let mut background = RED;
                world.was_pressed = true;
                let (x, y) = mouse_position();
                let pos = Vec2::new(x - (world.width / 2.0), y - (world.height / 2.0));
                let angle = cartesian2polar(&pos).angle.to_degrees().rem_euclid(360.0);
                if self.min < self.max {
                    if angle >= self.min && angle <= self.max {
                        world.score += 1;
                        background = WHITE;
                    }
                } else {
                    if angle > 180.0 {
                        if angle >= self.min && angle <= 360.0 {
                            world.score += 1;
                            background = WHITE;
                        }
                    } else {
                        if angle >= 0.0 && angle <= self.max {
                            world.score += 1;
                            background = WHITE;
                        }
                    }
                }
                world.count += 1;
                world.timer_start = Instant::now();
                let until = Duration::from_millis(world.config.wait_time);
                return SceneSwitch::replace(Waiting { until, background });
            }

            if !is_pressed {
                world.was_pressed = false;
            }
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data) {
        draw_score(world);
        let x = world.width / 2.0;
        let y = world.height / 2.0;
        let center = Vec2::new(x, y);

        let p2 = polar2cartesian(&center, 2000.0, self.min.to_radians());
        let p3 = polar2cartesian(&center, 2000.0, self.max.to_radians());
        draw_triangle(center, p2, p3, BLUE);

        // let t = Mesh::new_polygon(ctx, DrawMode::fill(), &[center, p2, p3], BLUE)?;
        // graphics::draw(ctx, &t, DrawParam::default())?;
        // let t = graphics::Text::new(format!("{} / {}", world.score, world.count));
        // graphics::draw(ctx, &t, (Point2 { x: 10.0, y: 10.0 }, BLACK));
    }
}

struct Fin;

impl Scene<Data, ()> for Fin {
    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        if is_mouse_button_pressed(MouseButton::Left) {
            world.count = 0;
            world.score = 0;
            world.was_pressed = false;
            SceneSwitch::replace(Waiting {
                until: Duration::from_millis(world.config.wait_time),
                background: WHITE,
            })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data) {
        draw_text(
            &format!(
                "You hit {} out of {}! Click to restart!",
                world.score,
                world.count - 1
            ),
            world.width / 2.0,
            world.height / 2.0,
            FONT_FIN,
            BLACK,
        );
    }
}

pub fn f64_to_duration(t: f64) -> Duration {
    debug_assert!(t > 0.0, "f64_to_duration passed a negative number!");
    let seconds = t.trunc();
    let nanos = t.fract() * 1e9;
    Duration::new(seconds as u64, nanos as u32)
}

fn fps_as_duration(fps: u32) -> Duration {
    let target_dt_seconds = 1.0 / f64::from(fps);
    f64_to_duration(target_dt_seconds)
}

/// A structure that contains our time-tracking state.
#[derive(Debug)]
pub struct TimeContext {
    last_instant: Instant,
    residual_update_dt: Duration,
    frame_count: usize,
}

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        TimeContext {
            last_instant: Instant::now(),
            residual_update_dt: Duration::from_secs(0),
            frame_count: 0,
        }
    }

    /// Update the state of the `TimeContext` to record that
    /// another frame has taken place.  Necessary for the FPS
    /// tracking and [`check_update_time()`](fn.check_update_time.html)
    /// functions to work.
    ///
    /// It's usually not necessary to call this function yourself,
    /// [`event::run()`](../event/fn.run.html) will do it for you.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let time_since_last = now - self.last_instant;
        self.last_instant = now;
        self.frame_count += 1;

        self.residual_update_dt += time_since_last;
    }
}

impl Default for TimeContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_update_time(timedata: &mut TimeContext, target_fps: u32) -> bool {
    let target_dt = fps_as_duration(target_fps);
    if timedata.residual_update_dt > target_dt {
        timedata.residual_update_dt -= target_dt;
        true
    } else {
        false
    }
}

#[macroquad::main("GoresTrainer")]
#[cfg(not(target_arch = "wasm32"))]
async fn main() {
    let mut my_game = MyGame::new();
    let mut timedata = TimeContext::new();
    loop {
        timedata.tick();
        while check_update_time(&mut timedata, FPS) {
            my_game.state.world.width = screen_width();
            my_game.state.world.height = screen_height();
            my_game.state.update();
        }
        clear_background(WHITE);
        my_game.state.draw();
        next_frame().await
    }
}

#[macroquad::main("GoresTrainer")]
#[cfg(target_arch = "wasm32")]
async fn main() {
    let mut my_game = MyGame::new();
    loop {
        my_game.state.world.width = screen_width();
        my_game.state.world.height = screen_height();
        my_game.state.update();
        clear_background(WHITE);
        my_game.state.draw();
        next_frame().await
    }
}

#[derive(Default)]
struct MyGame {
    state: SceneStack<Data, ()>,
}

impl MyGame {
    pub fn new() -> MyGame {
        // Load/create resources such as images here.
        let mut g = MyGame::default();
        g.state.push(Box::new(Start));
        g
    }
}

fn polar2cartesian(center: &Vec2, length: f32, angle: f32) -> Vec2 {
    let x = length * angle.cos();
    let y = length * angle.sin();
    Vec2::new(center.x + x, center.y + y)
}

#[derive(Default)]
pub struct Polar {
    pub len: f32,
    pub angle: f32,
}

fn cartesian2polar(cart_vec: &Vec2) -> Polar {
    let len = (cart_vec.x.powi(2) + cart_vec.y.powi(2)).sqrt();
    let angle = cart_vec.y.atan2(cart_vec.x);
    Polar { len, angle }
}
