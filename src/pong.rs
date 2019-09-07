use super::ball::Ball;
use super::camera::Camera;
use super::paddle::Paddle;
use super::phases::{maps_playfield, plays_pong, reads_controllers, Phase};
use ggez::event::EventHandler;
use ggez::graphics::Drawable;
use ggez::nalgebra::Point2;
use ggez::{graphics, Context, GameResult};
use rand::rngs::ThreadRng;

/// Game state that glues all parts of the game together.
pub struct Pong {
    /// Player's paddles.
    pub paddles: [Paddle; 2],

    /// Which phase is the game currently in. This is useful for view switching.
    pub phase: Phase,

    /// Input interface.
    pub camera: Camera,

    /// First float represents balls x coordinate, second float balls
    pub ball: Ball,

    /// Thread for rand create to generate random numbers.
    pub rand: ThreadRng,
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
            phase: Phase::MapsPlayfield,
            rand: ThreadRng::default(),
        }
    }
}

impl EventHandler for Pong {
    /// Update the game state or transitions into a new phase.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        match self.phase {
            Phase::MapsPlayfield => maps_playfield::update(self),
            Phase::ReadsController => reads_controllers::update(self),
            Phase::PlaysPong => plays_pong::update(self),
        }
    }

    /// Redraws the GUI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        match self.phase {
            Phase::MapsPlayfield => maps_playfield::draw(self, ctx),
            Phase::ReadsController => reads_controllers::draw(self, ctx),
            Phase::PlaysPong => plays_pong::draw(self, ctx),
        }?;

        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, xrel: f32, yrel: f32) {
        (*self.camera.positions[0].lock().unwrap()) = x;
        (*self.camera.positions[1].lock().unwrap()) = x;
    }
}
