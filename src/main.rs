extern crate ggez;

mod ball;
mod camera;
mod phase;
mod pong;

use ggez::event;
use ggez::ContextBuilder;
use pong::Pong;

fn main() {
    let (mut ctx, mut events) = ContextBuilder::new("pong", "Michael Bausano")
        .build()
        .expect("Could not create ggez context.");

    let mut game = Pong::new(&mut ctx);

    match event::run(&mut ctx, &mut events, &mut game) {
        Ok(_) => println!("Good game."),
        Err(e) => println!("Error occured: {}", e),
    }
}
