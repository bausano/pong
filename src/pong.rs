use super::ball::Ball;
use super::camera::Camera;
use super::paddle::Paddle;
use super::phases::Phase;
use ggez::event::EventHandler;
use ggez::{graphics, Context, GameResult};
use rand::rngs::ThreadRng;

/// Game state that glues all parts of the game together.
pub struct Pong {
    /// Player's paddles.
    pub paddles: [Paddle; 2],

    /// Which phase is the game currently in. This is useful for view switching.
    pub phase: Phase,

    /// Input interface. We only need the camera during the set up phase, when
    /// the game starts we can disown camera object to go do work in its own
    /// thread.
    pub camera: Option<Camera>,

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
            camera: Some(camera),
            ball: Default::default(),
            // Count downs 3 times one second before taking a picture of the
            // playfield.
            phase: Phase::MapsPlayfield { count_down: 3 },
            rand: ThreadRng::default(),
        }
    }
}

impl EventHandler for Pong {
    /// Update the game state or transitions into a new phase.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Phase::update(self)
    }

    /// Redraws the GUI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        Phase::draw(self, ctx)?;
        graphics::present(ctx)
    }

    /// The paddles can be also controlled by mouse.
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        _y: f32,
        _xrel: f32,
        _yrel: f32,
    ) {
        (*self.paddles[0].x.lock().unwrap()) = x as u32;
        // (*self.paddles[1].x.lock().unwrap()) = x as u32;
    }
}
