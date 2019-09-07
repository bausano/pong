use super::super::Pong;
use super::Phase;
use ggez::graphics::DrawParam;
use ggez::{Context, GameResult};

pub fn update(state: &mut Pong) -> GameResult<()> {
    state.phase = Phase::PlaysPong;

    Ok(())
}

pub fn draw(state: &mut Pong, ctx: &mut Context) -> GameResult<()> {
    Ok(())
}
