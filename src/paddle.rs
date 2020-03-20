use super::SCREEN_SIZE;
use ggez::graphics::{
    draw, BlendMode, Color, DrawMode, DrawParam, Drawable, MeshBuilder, Rect, BLACK,
};
use ggez::{Context, GameResult};
use std::sync::{Arc, Mutex};

/// Default paddle size. (width, height)
pub const PADDLE_SIZE: (f32, f32) = (50.0, 10.0);

pub struct Paddle {
    /// Which player controlls the paddle.
    pub player_id: u8,

    /// Width of the rectangle representing the paddle.
    pub width: f32,

    /// Height of the rectangle representing the paddle.
    pub height: f32,

    /// How many times has the paddle fail to bounce the ball.
    pub deaths: usize,

    /// Paddle colour will be used to draw the ball in the next tick.
    pub color: Color,

    /// Pointer to the mutex which updates the paddle's position.
    pub x: Arc<Mutex<u32>>,
}

impl Paddle {
    /// Spawns new player
    pub fn new(player_id: u8, x: Arc<Mutex<u32>>) -> Self {
        Paddle {
            x,
            player_id,
            deaths: 0,
            color: BLACK,
            width: PADDLE_SIZE.0,
            height: PADDLE_SIZE.1,
        }
    }

    /// Returns position of the top left corner of the paddle.
    pub fn position(&self) -> (f32, f32) {
        (
            ((*self.x.lock().unwrap()) as f32 - self.width / 2.0)
                .min(SCREEN_SIZE.0 - self.width)
                .max(0.0),
            self.player_id as f32 * (SCREEN_SIZE.1 - self.height),
        )
    }
}

impl Drawable for Paddle {
    /// Draws the ball on the canvas.
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let (paddle_x, paddle_y) = self.position();

        let paddle_shape = Rect::new(paddle_x, paddle_y, self.width, self.height);

        let paddle_mesh = MeshBuilder::new()
            .rectangle(DrawMode::fill(), paddle_shape, self.color)
            .build(ctx)?;

        draw(ctx, &paddle_mesh, param)
    }

    /// Creates a bounding box around the paddle..
    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        let (paddle_x, paddle_y) = self.position();

        Some(Rect::new(
            paddle_x,
            paddle_y,
            paddle_x + self.width,
            paddle_y + self.height,
        ))
    }

    /// Used to override a blend mode. In the case of the paddle,
    /// it is hard coded so this method is empty.
    fn set_blend_mode(&mut self, _: Option<BlendMode>) {
        //
    }

    /// Default graphics blend mode allows us to have the ball always on top.
    fn blend_mode(&self) -> Option<BlendMode> {
        None
    }
}
