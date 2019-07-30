pub struct Ball {
    /// The x and y coordinate of the ball's center.
    center: (f64, f64),

    /// The ball size.
    radius: f64,

    /// How fast is the ball moving in its direction.
    velocity: f64,

    /// The direction vector.
    direction: (f64, f64),
}

impl Default for Ball {
    /// Some default values which are going to be changed with the update for ball skins.
    fn default() -> Self {
        Ball {
            center: (0.0, 0.0),
            radius: 30.0,
            velocity: 5.0,
            direction: (0.0, 1.0),
        }
    }
}
