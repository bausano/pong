/// Classifies which phase is the game currently at. The enum is important for a
/// correct game flow and camera calibration.
pub enum Phase {
    /// In this stage of the game a square is rendered. The players are instructed
    /// to put the object they wish to control the game with in the middle. The
    /// object, for example a finger, is scanned by the camera.
    ReadsController,
}
