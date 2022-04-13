#![allow(unused_imports)]

use ggez::conf::{FullscreenType, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, FillOptions, Mesh};
use ggez::input::mouse;
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameResult};
use rand::prelude::ThreadRng;
use rand::Rng;

fn main() {
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("gores_trainer", "IAmHardliner")
        // .window_mode(WindowMode::fullscreen_type(WindowMode::default(), FullscreenType::Desktop))
        .window_setup(WindowSetup::default().title("Gores Aim Trainer"))
        .build()
        .expect("Failed to create context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, my_game);
}

#[derive(Debug)]
struct Target {
    pub min: f32,
    pub max: f32,
}

struct MyGame {
    pub was_pressed: bool,
    pub score: u32,
    pub count: u32,
    pub target: Option<Target>,
    pub rng: ThreadRng,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        MyGame {
            target: None,
            rng: rand::thread_rng(),
            score: 0,
            count: 0,
            was_pressed: false,
        }
    }
}

fn polar2cartesian(center: &Point2<f32>, length: f32, angle: f32) -> Point2<f32> {
    let x = length * angle.cos();
    let y = length * angle.sin();
    Point2 {
        x: center.x + x,
        y: center.y + y,
    }
}

#[derive(Default)]
pub struct Polar<T: Default> {
    pub len: T,
    pub angle: T,
}

fn cartesian2polar(cart_vec: &Point2<f32>) -> Polar<f32> {
    let len = (cart_vec.x.powi(2) + cart_vec.y.powi(2)).sqrt();
    let angle = cart_vec.y.atan2(cart_vec.x);
    Polar { len, angle }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, 60) {
            let window = graphics::window(ctx).inner_size();
            let width = window.width as f32;
            let height = window.height as f32;

            let is_pressed = mouse::button_pressed(ctx, event::MouseButton::Left);
            if let Some(target) = &self.target {
                if is_pressed && self.was_pressed == false {
                    self.was_pressed = true;
                    let pos = mouse::position(ctx);
                    let pos = Point2 {
                        x: pos.x - (width / 2.0),
                        y: pos.y - (height / 2.0),
                    };
                    let angle = cartesian2polar(&pos).angle.to_degrees().rem_euclid(360.0);
                    if target.min < target.max {
                        if angle >= target.min && angle <= target.max {
                            self.score += 1;
                        }
                    } else {
                        if angle > 180.0 {
                            if angle >= target.min && angle <= 360.0 {
                                self.score += 1;
                            }
                        } else {
                            if angle >= 0.0 && angle <= target.max {
                                self.score += 1;
                            }
                        }
                    }
                    self.target = None;
                    self.count += 1;
                }
            } else {
                let v = self.rng.gen_range(0..360u32) as f32;
                self.target = Some(Target {
                    min: (v - 10.0),
                    max: (v + 10.0),
                });
            }
            if !is_pressed {
                self.was_pressed = false;
            }
        }
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);
        if let Some(target) = &self.target {
            let window = graphics::window(ctx).inner_size();
            let width = window.width as f32;
            let height = window.height as f32;
            let center = Point2 {
                x: width / 2.0,
                y: height / 2.0,
            };
            let p2 = polar2cartesian(&center, 2000.0, target.min.to_radians());
            let p3 = polar2cartesian(&center, 2000.0, target.max.to_radians());
            let t = Mesh::new_polygon(ctx, DrawMode::fill(), &[center, p2, p3], Color::BLUE)?;
            graphics::draw(ctx, &t, DrawParam::default())?;
        }
        let t = graphics::Text::new(format!("{} / {}", self.score, self.count));
        graphics::draw(ctx, &t, (Point2 { x: 10.0, y: 10.0 }, Color::BLACK))?;
        // Draw code here...
        graphics::present(ctx)
    }
}
