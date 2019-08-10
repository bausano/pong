extern crate ggez;
extern crate rand;

mod ball;
mod camera;
mod phase;
mod pong;

use ggez::event;
use ggez::ContextBuilder;
use pong::Pong;

/// How large should the game window be in pixels.
const SCREEN_SIZE: (f32, f32) = (500.0, 500.0);

fn main() {
    let (mut ctx, mut events) = ContextBuilder::new("pong", "Michael Bausano")
        .window_setup(ggez::conf::WindowSetup::default().title("Pong"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .expect("Could not create ggez context.");

    let mut game = Pong::new(&mut ctx);

    match event::run(&mut ctx, &mut events, &mut game) {
        Ok(_) => println!("Good game."),
        Err(e) => println!("Error occured: {}", e),
    }
}
