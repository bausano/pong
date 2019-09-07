pub mod maps_playfield;
pub mod plays_pong;
pub mod reads_controllers;

/// Classifies which phase is the game currently at. The enum is important for a
/// correct game flow and camera calibration.
pub enum Phase {
    /// Renders cubes in top right and bottom left corners of the screen to map the
    /// projected screen output to the camera input.
    MapsPlayfield,

    /// In this stage of the game a square is rendered. The players are instructed
    /// to put the object they wish to control the game with in the middle. The
    /// object, for example a finger, is scanned by the camera.
    ReadsController,

    /// In this stage the camera module is set up for both players and the game
    /// begins by rendering the ball.
    PlaysPong,
}
