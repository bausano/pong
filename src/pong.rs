use super::ball::Ball;
use super::camera::Camera;
use super::phase::Phase;
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
        println!("Update");

        Ok(())
    }

    /// Redraws the GUI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r1 =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;

        graphics::draw(ctx, &r1, graphics::DrawParam::default())?;

        graphics::present(ctx)
    }
}
