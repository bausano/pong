mod maps_playfield;
use crate::pong::Pong;
use ggez::{Context, GameResult};
mod plays_pong;

/// Classifies which phase is the game currently at. The enum is important for a
/// correct game flow and camera calibration.
pub enum Phase {
    /// Renders cubes in top right and bottom left corners of the screen to map the
    /// projected screen output to the camera input.
    MapsPlayfield {
        // How long until the camera takes a capture of the playfield.
        count_down: usize,
    },

    /// In this stage the camera module is set up for both players and the game
    /// begins by rendering the ball.
    PlaysPong,
}

impl Phase {
    pub fn update(state: &mut Pong) -> GameResult<()> {
        match state.phase {
            Phase::MapsPlayfield { .. } => maps_playfield::update(state),
            Phase::PlaysPong => plays_pong::update(state),
        }
    }
    pub fn draw(state: &mut Pong, ctx: &mut Context) -> GameResult<()> {
        match state.phase {
            Phase::MapsPlayfield { .. } => maps_playfield::draw(state, ctx),
            Phase::PlaysPong => plays_pong::draw(state, ctx),
        }
    }
}
