use super::Phase;
use crate::pong::Pong;
use ggez::{Context, GameResult};
use std::thread;
use std::time::Duration;

pub fn update(state: &mut Pong) -> GameResult<()> {
    if let Phase::MapsPlayfield { ref mut count_down } = state.phase {
        if *count_down == 0 {
            info!("Taking a snapshot of the playfield before the game.");
            state
                .camera
                .as_mut()
                .expect("The game has not begun yet, the camera object must be present.")
                .map_playfield();
            state.phase = Phase::PlaysPong;
            state.camera.take().unwrap().start_capturing();
        } else {
            debug!("Will take a snapshot of the field in {}", count_down);
            *count_down -= 1;
        }
        thread::sleep(Duration::from_secs(1));
    } else {
        unreachable!("Run logic for mapping a playfield with wrong phase.");
    }

    Ok(())
}

pub fn draw(_state: &mut Pong, _ctx: &mut Context) -> GameResult<()> {
    Ok(())
}
