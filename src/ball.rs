use super::paddle::Paddle;
use super::WINDOW_SIZE;
use ggez::graphics::{
    draw, BlendMode, Color, DrawMode, DrawParam, Drawable, MeshBuilder, Rect,
    BLACK,
};
use ggez::nalgebra::Point2;
use ggez::{Context, GameResult};
use rand::rngs::ThreadRng;
use rand::Rng;

pub const RADIUS: f32 = 13.0;
pub const INCREMENT_FACTOR: f32 = 1.0 / 500.0;
pub const DECREMENT_FACTOR: f32 = 1.0 / 500.0;
pub const RANDOM_BOUNCE_BOUND: f32 = 0.05;
pub const WALL_ACCELERATION_BONUS: f32 = 1.4;
pub const PADDLE_ACCELERATION_BONUS: f32 = 1.9;
pub const MAX_ACCELERATION: f32 = 2.3;
pub const MIN_ACCELERATION: f32 = -5.0;
pub const MAX_VELOCITY: f32 = 6.0;
pub const MIN_VELOCITY: f32 = 4.5;

pub struct Ball {
    /// The x and y coordinate of the ball's center.
    pub center: (f32, f32),

    /// The ball size.
    pub radius: f32,

    /// The ball's colour which will be used to draw the ball in the next tick.
    pub color: Color,

    /// How fast is the ball moving in its direction.
    velocity: f32,

    /// The ball will gain one fifth of the current acceleration as velocity
    /// each tick and the acceleration will decrease accordingly.
    acceleration: f32,

    /// The direction vector. Both x and y are in interval <-1; 1>.
    /// ```
    ///        1|  v
    ///         | /
    ///    -1   |/
    ///    -----+----- 1
    ///         |
    ///         |
    ///         |-1
    /// ```
    /// In the example above, vector `v` = (0.5, 1). That means that in
    /// the next tick, its new position will be changed accordingly to
    /// this direction scaled by the velocity.
    direction: (f32, f32),
}

impl Default for Ball {
    /// Some default values which are going to be changed with the update for ball skins.
    fn default() -> Self {
        Ball {
            center: (WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / 2.0),
            radius: RADIUS,
            velocity: 5.0,
            acceleration: 0.0,
            direction: (1.0, 0.15),
            color: BLACK,
        }
    }
}

impl Ball {
    /// First checks the balls position. If the ball gets close
    /// to a wall, it bounces the ball off. Then based on current
    /// acceleration increases or decreases ball speed.
    pub fn tick(&mut self, rng: &mut ThreadRng) {
        self.bounce_from_wall(rng);

        // If the acceleration would slow down the ball, sets the acceleration
        // bonus to zero and slowly starts decrementing the velocity.
        if self.acceleration < 1.0 {
            self.acceleration = 0.0;

            self.velocity = MIN_VELOCITY
                .max(self.velocity * DECREMENT_FACTOR)
                .min(MAX_VELOCITY);
        } else {
            // Increments the velocity and decrements the acceleration.
            let increment = self.acceleration * INCREMENT_FACTOR;
            self.acceleration -= increment;
            self.velocity += increment;
        }

        self.center.0 += self.velocity * self.direction.0;
        self.center.1 += self.velocity * self.direction.1;
    }

    /// Checks whether the ball missed user paddle and hit
    /// a wall perpendicular to the y axis. If so, returns
    /// id of the player
    ///
    /// ```
    /// | Player 0 |
    /// |          |
    /// | -------- |
    /// |          |
    /// | Player 1 |
    /// ```
    ///
    pub fn player_scored(&mut self) -> Option<u8> {
        if self.center.1 >= WINDOW_SIZE.1 {
            return Some(0);
        }

        if self.center.1 <= 0.0 {
            return Some(1);
        }

        None
    }

    /// Checks whether the ball should bounce from a wall
    /// parallel to the y axis.
    pub fn bounce_from_wall(&mut self, rng: &mut ThreadRng) {
        // If the ball is touching or is beyond the right wall and its direction is to the right as
        // well (positive x value of the direction vector), then the ball should bounce.
        let bounces_off: bool = self.center.0 + self.radius >= WINDOW_SIZE.0
            && self.direction.0 >= 0.0;

        // Similar check is applied for the left wall.
        let bounces_off: bool = bounces_off
            || (self.center.0 - self.radius <= 0.0 && self.direction.0 <= 0.0);

        if bounces_off {
            // Bounces off a vertical collider - a wall.
            self.bounce((-1.0, 1.0), WALL_ACCELERATION_BONUS, rng);
        }
    }

    pub fn bounce_from_paddle(&mut self, paddle: &Paddle, rng: &mut ThreadRng) {
        if paddle.player_id == 0 && self.direction.1 > 0.0 {
            return;
        }
        if paddle.player_id == 1 && self.direction.1 < 0.0 {
            return;
        }

        let (left_x, top_y) = paddle.position();
        let right_x = left_x + paddle.width;
        let bottom_y = top_y + paddle.height;

        if self.contains(left_x, top_y)
            || self.contains(right_x, top_y)
            || self.contains(left_x, bottom_y)
            || self.contains(right_x, bottom_y)
        {
            return self.bounce((-1.0, -1.0), PADDLE_ACCELERATION_BONUS, rng);
        }

        // If the paddle is player one's, but the ball is above the paddle, return,
        if paddle.player_id == 0 && self.center.1 - self.radius > bottom_y {
            return;
        }

        // If the paddle is player two's, but the ball is below the paddle, return.
        if paddle.player_id == 1 && self.center.1 + self.radius < top_y {
            return;
        }

        if self.center.0 + self.radius >= left_x
            && self.center.0 - self.radius <= right_x
        {
            return self.bounce((1.0, -1.0), PADDLE_ACCELERATION_BONUS, rng);
        }
    }

    /// Applies transformation to ball's direction vector. The new direction vector is always
    /// in interval <-1; 1> for both x and y.
    pub fn bounce(
        &mut self,
        (x, y): (f32, f32),
        accelerate: f32,
        rng: &mut ThreadRng,
    ) {
        // Increases acceleration but keeps it between thresholds. The ball will gain N of its
        // acceleration as velocity every tick.
        self.acceleration = (self.acceleration + accelerate)
            .min(MAX_ACCELERATION)
            .max(MIN_ACCELERATION);

        // Small nudge in a random direction.
        let random_bounce =
            rng.gen_range(-RANDOM_BOUNCE_BOUND, RANDOM_BOUNCE_BOUND);

        // Applies the transformation vector.
        let new_x = self.direction.0 * x + random_bounce;
        let new_y = self.direction.1 * y + random_bounce;

        // Puts both direction in positive and selects the greater one.
        let max = new_x.abs().max(new_y.abs());

        // Normalizes the vector into <-1; 1> interval.
        self.direction.0 = new_x / max;
        self.direction.1 = new_y / max;
    }

    /// Whether the ball contains given point.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        (x - self.center.0).powi(2) + (y - self.center.1).powi(2)
            <= self.radius.powi(2)
    }
}

impl Drawable for Ball {
    /// Draws the ball on the canvas.
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let center = Point2::new(self.center.0, self.center.1);

        let ball_mesh = MeshBuilder::new()
            .circle(DrawMode::fill(), center, self.radius, 1.0, self.color)
            .build(ctx)?;

        draw(ctx, &ball_mesh, param)
    }

    /// Creates a bounding box around the ball.
    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(Rect::new(
            self.center.0 - self.radius,
            self.center.1 - self.radius,
            self.radius * 2.0,
            self.radius * 2.0,
        ))
    }

    /// Used to override a blend mode. In the case of the circle,
    /// it is hard coded so this method is empty.
    fn set_blend_mode(&mut self, _: Option<BlendMode>) {
        //
    }

    /// Sets the ball to be always on top.
    fn blend_mode(&self) -> Option<BlendMode> {
        Some(BlendMode::Replace)
    }
}
