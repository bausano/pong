#[macro_use]
extern crate log;

mod ball;
mod camera;
mod paddle;
mod phases;
mod pong;

use ggez::event;
use ggez::ContextBuilder;
use pong::Pong;

/// How large should the game window be in pixels.
const WINDOW_SIZE: (f32, f32) = (1200.0, 700.0);

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let (mut ctx, mut events) = ContextBuilder::new("pong", "Michael Bausano")
        .window_setup(ggez::conf::WindowSetup::default().title("Pong"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1),
        )
        .build()
        .expect("Could not create ggez context.");

    let mut game = Pong::new(&mut ctx);
    match event::run(&mut ctx, &mut events, &mut game) {
        Ok(_) => info!("Good game."),
        Err(e) => error!("Error occured: {}", e),
    }
}
