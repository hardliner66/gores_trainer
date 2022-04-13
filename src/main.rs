use ggez::conf::WindowSetup;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh};
use ggez::input::mouse;
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameResult};
use rand::prelude::ThreadRng;
use rand::Rng;

mod scene;
use scene::*;

const FPS: u32 = 60;

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
    fn update(&mut self, _world: &mut Data, ctx: &mut ggez::Context) -> SceneSwitch<Data, ()> {
        if mouse::button_pressed(ctx, event::MouseButton::Left) {
            SceneSwitch::replace(Waiting {
                ticks: 60,
                background: Color::WHITE,
            })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let t = graphics::Text::new("Click to start!");
        graphics::draw(
            ctx,
            &t,
            (
                Point2 {
                    x: world.width / 2.0,
                    y: world.height / 2.0,
                },
                Color::BLACK,
            ),
        )
    }
}

pub struct Waiting {
    pub ticks: u32,
    pub background: Color,
}

impl Scene<Data, ()> for Waiting {
    fn draw(&mut self, world: &mut Data, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, self.background);
        let t = graphics::Text::new(format!("{} / {}", world.score, world.count));
        graphics::draw(ctx, &t, (Point2 { x: 10.0, y: 10.0 }, Color::BLACK))
    }

    fn update(&mut self, world: &mut Data, _ctx: &mut ggez::Context) -> SceneSwitch<Data, ()> {
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
    fn update(&mut self, world: &mut Data, ctx: &mut ggez::Context) -> SceneSwitch<Data, ()> {
        self.ticks -= 1;
        if self.ticks <= 0 {
            world.count += 1;
            let ticks = world.rng.gen_range(1..=6) * FPS / 2;
            SceneSwitch::replace(Waiting {
                ticks,
                background: Color::RED,
            })
        } else {
            let is_pressed = mouse::button_pressed(ctx, event::MouseButton::Left);
            if is_pressed && world.was_pressed == false {
                let mut background = Color::RED;
                world.was_pressed = true;
                let pos = mouse::position(ctx);
                let pos = Point2 {
                    x: pos.x - (world.width / 2.0),
                    y: pos.y - (world.height / 2.0),
                };
                let angle = cartesian2polar(&pos).angle.to_degrees().rem_euclid(360.0);
                if self.min < self.max {
                    if angle >= self.min && angle <= self.max {
                        world.score += 1;
                        background = Color::WHITE;
                    }
                } else {
                    if angle > 180.0 {
                        if angle >= self.min && angle <= 360.0 {
                            world.score += 1;
                            background = Color::WHITE;
                        }
                    } else {
                        if angle >= 0.0 && angle <= self.max {
                            world.score += 1;
                            background = Color::WHITE;
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

    fn draw(&mut self, world: &mut Data, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let window = graphics::window(ctx).inner_size();
        let width = window.width as f32;
        let height = window.height as f32;
        let center = Point2 {
            x: width / 2.0,
            y: height / 2.0,
        };
        let p2 = polar2cartesian(&center, 2000.0, self.min.to_radians());
        let p3 = polar2cartesian(&center, 2000.0, self.max.to_radians());
        let t = Mesh::new_polygon(ctx, DrawMode::fill(), &[center, p2, p3], Color::BLUE)?;
        graphics::draw(ctx, &t, DrawParam::default())?;
        let t = graphics::Text::new(format!("{} / {}", world.score, world.count));
        graphics::draw(ctx, &t, (Point2 { x: 10.0, y: 10.0 }, Color::BLACK))
    }
}

struct Fin;

impl Scene<Data, ()> for Fin {
    fn update(&mut self, _world: &mut Data, ctx: &mut ggez::Context) -> SceneSwitch<Data, ()> {
        if mouse::button_pressed(ctx, event::MouseButton::Left) {
            SceneSwitch::replace(Waiting {
                ticks: 60,
                background: Color::WHITE,
            })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut Data, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let t = graphics::Text::new(format!(
            "You hit {} out of {}! Click to restart!",
            world.score,
            world.count - 1
        ));
        graphics::draw(
            ctx,
            &t,
            (
                Point2 {
                    x: world.width / 2.0,
                    y: world.height / 2.0,
                },
                Color::BLACK,
            ),
        )
    }
}

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

#[derive(Default)]
struct MyGame {
    state: SceneStack<Data, ()>,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        let mut g = MyGame::default();
        g.state.push(Box::new(Start));
        g
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
        while ggez::timer::check_update_time(ctx, FPS) {
            let window = graphics::window(ctx).inner_size();
            self.state.world.width = window.width as f32;
            self.state.world.height = window.height as f32;
            self.state.update(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);
        self.state.draw(ctx);
        graphics::present(ctx)
    }
}
