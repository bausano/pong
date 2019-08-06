use super::SCREEN_SIZE;

pub const MAX_ACCELERATION: f32 = 15.0;

pub struct Ball {
    /// The x and y coordinate of the ball's center.
    pub center: (f32, f32),

    /// The ball size.
    pub radius: f32,

    /// How fast is the ball moving in its direction.
    velocity: f32,

    /// The ball will gain one fifth of the current acceleration as velocity
    /// each tick and the acceleration will decrease accordingly.
    acceleration: f64,

    /// The direction vector.
    direction: (f32, f32),
}

impl Default for Ball {
    /// Some default values which are going to be changed with the update for ball skins.
    fn default() -> Self {
        Ball {
            center: (SCREEN_SIZE.0, SCREEN_SIZE.1),
            radius: 30.0,
            velocity: 5.0,
            acceleration: 0.0,
            direction: (0.0, 1.0),
        }
    }
}

impl Ball {
    /// Checks whether the ball missed user paddle and hit
    /// a wall perpendicular to the y axis. If so, returns
    /// id of the player
    ///
    /// | Player 0 |
    /// |          |
    /// | -------- |
    /// |          |
    /// | Player 1 |
    ///
    pub fn player_scored(&mut self) -> Option<u8> {
        if self.center.1 + self.radius >= SCREEN_SIZE.1 {
            return Some(0);
        }

        if self.center.1 - self.radius <= 0.0 {
            return Some(1);
        }

        None
    }

    /// Checks whether the ball should bounce from a wall
    /// parallel to the y axis.
    pub fn bounce_from_wall(&mut self) {
        if self.center.0 + self.radius >= SCREEN_SIZE.0 {
            // Bounce from the right wall.
            return;
        }

        // If the ball did not hit the left wall, return.
        if self.center.0 - self.radius > 0.0 {
            return;
        }

        // Bounce from the left wall.
    }

    pub fn bounce(&mut self, (x, y): (f32, f32), accelerate: f32) {
        //
        self.acceleration = (self.acceleration + accelerate).min(MAX_ACCELERATION);
    }
}
