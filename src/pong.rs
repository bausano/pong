use super::ball::Ball;
use super::camera::Camera;
use super::paddle::Paddle;
use super::phase::Phase;
use ggez::event::EventHandler;
use ggez::graphics::Drawable;
use ggez::nalgebra::Point2;
use ggez::{graphics, Context, GameResult};
use rand::rngs::ThreadRng;

/// Game state that glues all parts of the game together.
pub struct Pong {
    /// Player's paddles.
    paddles: [Paddle; 2],

    /// Which phase is the game currently in. This is useful for view switching.
    phase: Phase,

    /// Input interface.
    camera: Camera,

    /// First float represents balls x coordinate, second float balls
    ball: Ball,

    /// Thread for rand create to generate random numbers.
    rand: ThreadRng,
}

impl Pong {
    /// Creates a new state object.
    pub fn new(_: &mut Context) -> Self {
        let camera = Camera::new();

        Pong {
            paddles: [
                Paddle::new(0, camera.positions[0].clone()),
                Paddle::new(1, camera.positions[1].clone()),
            ],
            camera,
            ball: Default::default(),
            phase: Phase::ReadsController,
            rand: ThreadRng::default(),
        }
    }
}

impl EventHandler for Pong {
    /// Update the game state or transition into a new phase.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if let Some(player) = self.ball.player_scored() {
            println!("Player {} scored.", player);

            self.ball = Default::default();
        }

        // Tries to bounce the ball from a wall if it's close enough.
        self.ball.bounce_from_wall(&mut self.rand);

        // Moves the ball.
        self.ball.tick();

        Ok(())
    }

    /// Redraws the GUI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        self.ball.draw(ctx, graphics::DrawParam::default())?;
        self.paddles[0].draw(ctx, graphics::DrawParam::default())?;
        self.paddles[1].draw(ctx, graphics::DrawParam::default())?;

        graphics::present(ctx)
    }
}
