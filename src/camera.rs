use super::SCREEN_SIZE;
use std::sync::{Arc, Mutex};

/// Object used for scanning the camera input. It is updated with relevant values such as
/// controller patterns (e.g. a finger) or the controller position.
pub struct Camera {
    /// Controller is a square or N pixels. If the size is 0 the camera has not been calibrated
    /// yet.
    controller_size: u32,

    /// A list of RGB values which are used to identify a controller for each player.
    /// There are two players in the basic version, therefore the size is 2.
    controllers: [Vec<(u8, u8, u8)>; 2],

    /// Latest position of each player. This value is updated by the camera.
    pub positions: [Arc<Mutex<f32>>; 2],
}

impl Camera {
    /// Builds a new empty camera that has be to calibrated.
    pub fn new() -> Self {
        Camera {
            controller_size: 0,
            controllers: [Vec::new(), Vec::new()],
            positions: [
                Arc::new(Mutex::new(SCREEN_SIZE.1 / 2.0)),
                Arc::new(Mutex::new(SCREEN_SIZE.1 / 2.0)),
            ],
        }
    }
}
