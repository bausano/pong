use super::super::{Pong, SCREEN_SIZE};
use super::Phase;
use ggez::graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect, BLACK, WHITE};
use ggez::{Context, GameResult};

pub fn update(state: &mut Pong) -> GameResult<()> {
    let mut camera = rscam::new("/dev/video2").unwrap();

    camera
        .start(&rscam::Config {
            interval: (1, 30),
            resolution: (1280, 720),
            format: b"RGB3",
            ..Default::default()
        })
        .unwrap();

    let mut rows: [u64; 720] = [0; 720];
    let mut cols: [u64; 1280] = [0; 1280];

    let frame = camera.capture().unwrap();

    for pixel in 0..(&frame.len() / 3) {
        let value: u64 = frame[(pixel * 3)..(pixel * 3 + 3)]
            .iter()
            .fold(0, |acc, x| acc + *x as u64);

        cols[pixel / 720] += value as u64;
        rows[pixel / 1280] += value as u64;
    }

    // println!("cols: ");
    // cols.iter().for_each(|x| print!("{} ", x / 720));
    println!("rows: \n");
    rows.iter().for_each(|x| print!("{} ", x / 1280));

    Ok(())
}

/// Draws two red rectangles spanning the whole width of the screen and one sixth of the height.
pub fn draw(state: &mut Pong, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, BLACK);

    let highlight_height = SCREEN_SIZE.1 / 6.0;

    let highlight_top = Rect::new(0.0, 0.0, SCREEN_SIZE.0, highlight_height);
    let highlight_bottom = Rect::new(
        0.0,
        SCREEN_SIZE.1 - highlight_height,
        SCREEN_SIZE.0,
        highlight_height,
    );

    let highlights = MeshBuilder::new()
        .rectangle(DrawMode::fill(), highlight_top, WHITE)
        .rectangle(DrawMode::fill(), highlight_bottom, WHITE)
        .build(ctx)?;

    graphics::draw(ctx, &highlights, DrawParam::default())?;

    Ok(())
}
