use super::ball::Ball;
use super::camera::Camera;
use super::phase::Phase;
use super::SCREEN_SIZE;
use ggez::event::EventHandler;
use ggez::{graphics, Context, GameResult};

/// Game state that glues all parts of the game together.
pub struct Pong {
    /// Which phase is the game currently in. This is useful for view switching.
    phase: Phase,

    /// Input interface.
    camera: Camera,

    /// First float represents balls x coordinate, second float balls
    ball: Ball,
}

impl Pong {
    /// Creates a new state object.
    pub fn new(_: &mut Context) -> Self {
        Pong {
            camera: Camera::new(),
            ball: Default::default(),
            phase: Phase::ReadsController,
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

        // Tries to bounce the ball from a wall.
        self.ball.bounce_from_wall();

        Ok(())
    }

    /// Redraws the GUI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        // TODO: Handle mutex error.
        // TODO: Draw the paddle once and the just update its position.
        for (center_x, center_y) in (*self.camera.positions.lock().unwrap()).iter() {
            // TODO: Make paddle size editable.
            let paddle_x = (center_x - 50.0).min(SCREEN_SIZE.0 - 100.0).max(50.0);
            let paddle_y = (center_y - 10.0).min(SCREEN_SIZE.1 - 20.0).max(10.0);

            let paddle_shape = graphics::Rect::new(paddle_x, paddle_y, 100.0, 20.0);
            let paddle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                paddle_shape,
                graphics::BLACK,
            )?;

            graphics::draw(ctx, &paddle, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)
    }
}
