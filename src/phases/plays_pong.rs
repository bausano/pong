use super::super::Pong;
use ggez::graphics::{DrawParam, Drawable};
use ggez::{Context, GameResult};

/// Updates the ball direction, velocity and position.
pub fn update(state: &mut Pong) -> GameResult<()> {
    if state.ball.player_scored().is_some() {
        state.ball = Default::default();
    }

    for ref paddle in state.paddles.iter() {
        state.ball.bounce_from_paddle(paddle, &mut state.rand);
    }

    // Moves the ball and bounces the ball from the wall if close enough.
    state.ball.tick(&mut state.rand);

    Ok(())
}

/// Redraws the game GUI elements: the two paddles and the ball.
pub fn draw(state: &mut Pong, ctx: &mut Context) -> GameResult<()> {
    state.ball.draw(ctx, DrawParam::default())?;
    state.paddles[0].draw(ctx, DrawParam::default())?;
    state.paddles[1].draw(ctx, DrawParam::default())?;

    Ok(())
}
