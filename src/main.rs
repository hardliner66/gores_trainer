use std::cmp;
use std::time;

use macroquad::prelude::*;

mod scene;
use ::rand::prelude::ThreadRng;
use ::rand::Rng;
use scene::*;

const FPS: u32 = 60;
const FONT_START: f32 = 30.0;
const FONT_FIN: f32 = 30.0;
const FONT_SCORE: f32 = 18.0;

#[derive(Default)]
pub struct Data {
    pub score: u32,
    pub count: u32,
    pub rng: ThreadRng,
    pub was_pressed: bool,
    pub width: f32,
    pub height: f32,
}

pub struct Start;

impl Scene<Data, ()> for Start {
    fn update(&mut self, _world: &mut Data) -> SceneSwitch<Data, ()> {
        if macroquad::input::is_mouse_button_pressed(MouseButton::Left) {
            SceneSwitch::replace(Waiting {
                ticks: 60,
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
    pub ticks: u32,
    pub background: Color,
}

impl Scene<Data, ()> for Waiting {
    fn draw(&mut self, world: &mut Data) {
        clear_background(self.background);
        draw_text(
            &format!("{} / {}", world.score, world.count),
            10.0,
            5.0 + FONT_SCORE,
            FONT_SCORE,
            BLACK,
        );
    }

    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        if world.count > 50 {
            SceneSwitch::replace(Fin);
        }
        self.ticks -= 1;
        if self.ticks <= 0 {
            let v = world.rng.gen_range(0..360u32) as f32;
            SceneSwitch::replace(Target {
                ticks: 1 * FPS,
                min: v - 5.0,
                max: v + 5.0,
            })
        } else {
            SceneSwitch::None
        }
    }
}

pub struct Target {
    pub ticks: u32,
    pub min: f32,
    pub max: f32,
}

impl Scene<Data, ()> for Target {
    fn update(&mut self, world: &mut Data) -> SceneSwitch<Data, ()> {
        self.ticks -= 1;
        if self.ticks <= 0 {
            world.count += 1;
            let ticks = world.rng.gen_range(1..=6) * FPS / 2;
            SceneSwitch::replace(Waiting {
                ticks,
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
                let ticks = world.rng.gen_range(1..=6) * FPS / 2;
                return SceneSwitch::replace(Waiting { ticks, background });
            }

            if !is_pressed {
                world.was_pressed = false;
            }
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data) {
        draw_text(
            &format!("{} / {}", world.score, world.count),
            10.0,
            10.0,
            FONT_SCORE,
            BLACK,
        );
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
    fn update(&mut self, _world: &mut Data) -> SceneSwitch<Data, ()> {
        if is_mouse_button_pressed(MouseButton::Left) {
            SceneSwitch::replace(Waiting {
                ticks: 60,
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

pub fn f64_to_duration(t: f64) -> time::Duration {
    debug_assert!(t > 0.0, "f64_to_duration passed a negative number!");
    let seconds = t.trunc();
    let nanos = t.fract() * 1e9;
    time::Duration::new(seconds as u64, nanos as u32)
}

fn fps_as_duration(fps: u32) -> time::Duration {
    let target_dt_seconds = 1.0 / f64::from(fps);
    f64_to_duration(target_dt_seconds)
}

/// A simple buffer that fills
/// up to a limit and then holds the last
/// N items that have been inserted into it,
/// overwriting old ones in a round-robin fashion.
///
/// It's not quite a ring buffer 'cause you can't
/// remove items from it, it just holds the last N
/// things.
#[derive(Debug, Clone)]
struct LogBuffer<T>
where
    T: Clone,
{
    head: usize,
    size: usize,
    /// The number of actual samples inserted, used for
    /// smarter averaging.
    samples: usize,
    contents: Vec<T>,
}

impl<T> LogBuffer<T>
where
    T: Clone + Copy,
{
    fn new(size: usize, init_val: T) -> LogBuffer<T> {
        LogBuffer {
            head: 0,
            size,
            contents: vec![init_val; size],
            // Never divide by 0
            samples: 1,
        }
    }

    /// Pushes a new item into the `LogBuffer`, overwriting
    /// the oldest item in it.
    fn push(&mut self, item: T) {
        self.head = (self.head + 1) % self.contents.len();
        self.contents[self.head] = item;
        self.size = cmp::min(self.size + 1, self.contents.len());
        self.samples += 1;
    }

    /// Returns a slice pointing at the contents of the buffer.
    /// They are in *no particular order*, and if not all the
    /// slots are filled, the empty slots will be present but
    /// contain the initial value given to [`new()`](#method.new).
    ///
    /// We're only using this to log FPS for a short time,
    /// so we don't care for the second or so when it's inaccurate.
    fn contents(&self) -> &[T] {
        if self.samples > self.size {
            &self.contents
        } else {
            &self.contents[..self.samples]
        }
    }

    /// Returns the most recent value in the buffer.
    fn latest(&self) -> T {
        self.contents[self.head]
    }
}

/// A structure that contains our time-tracking state.
#[derive(Debug)]
pub struct TimeContext {
    init_instant: time::Instant,
    last_instant: time::Instant,
    frame_durations: LogBuffer<time::Duration>,
    residual_update_dt: time::Duration,
    frame_count: usize,
}

/// How many frames we log update times for.
const TIME_LOG_FRAMES: usize = 200;

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        let initial_dt = time::Duration::from_millis(16);
        TimeContext {
            init_instant: time::Instant::now(),
            last_instant: time::Instant::now(),
            frame_durations: LogBuffer::new(TIME_LOG_FRAMES, initial_dt),
            residual_update_dt: time::Duration::from_secs(0),
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
        let now = time::Instant::now();
        let time_since_last = now - self.last_instant;
        self.frame_durations.push(time_since_last);
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
async fn main() {
    let mut my_game = MyGame::new();
    let mut time = TimeContext::new();
    loop {
        time.tick();
        while check_update_time(&mut time, FPS) {
            my_game.state.world.width = screen_width();
            my_game.state.world.height = screen_height();
            my_game.state.update();
        }
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
