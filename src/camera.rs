use super::SCREEN_SIZE;
use std::sync::{Arc, Mutex};
use std::thread;

const CAMERA_DEV: &str = "/dev/video2";
const RGB: &[u8] = b"RGB3";

/// Object used for scanning the camera input. It updates controller positions.
pub struct Camera {
    /// Latest position of each player. This value is updated by the camera.
    /// TODO: Consider making these atomic u32s.
    pub positions: [Arc<Mutex<u32>>; 2],

    /// Handle to the started camera which can capture images.
    handle: rscam::Camera,

    /// What was the average gray for each column of the top half when the game
    /// started.
    /// TODO: Name background.
    top_half_averages: [u8; 1280],

    /// What was the average gray for each column of the bottom half when the
    /// game started.
    bottom_half_averages: [u8; 1280],
}

impl Camera {
    /// Builds a new empty camera that has be to calibrated.
    pub fn new() -> Self {
        let mut handle =
            rscam::new(CAMERA_DEV).expect("Cannot find camera device");

        // Starts the camera, now we can capture images.
        handle
            .start(&rscam::Config {
                interval: (1, 30),
                resolution: (1280, 720),
                format: RGB,
                ..Default::default()
            })
            .expect("Cannot start camera");
        // Tests the camera.
        handle.capture().expect("Cannot take a capture");

        Self {
            positions: [
                Arc::new(Mutex::new(SCREEN_SIZE.1 as u32 / 2)),
                Arc::new(Mutex::new(SCREEN_SIZE.1 as u32 / 2)),
            ],
            handle,
            top_half_averages: [0; 1280],
            bottom_half_averages: [0; 1280],
        }
    }

    /// Captures the empty playfield to learn about its default colours and
    /// inconsistencies. It records this default state and when the camera
    /// thread starts updating the positions, it will calculate them against
    /// this default.
    pub fn map_playfield(&mut self) {
        // Capture the visible field.
        let frame = self.handle.capture().expect("Cannot capture camera input");

        // Update averages.
        average_gray_for_frame_halves(
            &frame,
            &mut self.top_half_averages,
            &mut self.bottom_half_averages,
        );
    }

    /// Starts a new thread on which the camera continuously screens the
    /// playfield and update the paddle positions.
    pub fn start_capturing(self) {
        info!("Starting new thread for camera.");
        thread::spawn(move || {
            let mut top_half = [0; 1280];
            let mut bottom_half = [0; 1280];
            loop {
                let frame =
                    self.handle.capture().expect("Cannot capture camera input");
                average_gray_for_frame_halves(
                    &frame,
                    &mut top_half,
                    &mut bottom_half,
                );

                // Finds both controllers in their respective halves and returns
                // their position on the x axis.
                let top_x =
                    find_controller(&self.top_half_averages, &mut top_half);
                let bottom_x = find_controller(
                    &self.bottom_half_averages,
                    &mut bottom_half,
                );

                // Updates the controllers.
                for x in top_x {
                    (*self.positions[0].lock().unwrap()) = x;
                }
                for x in bottom_x {
                    (*self.positions[1].lock().unwrap()) = x;
                }
            }
        });
    }
}

fn average_gray_for_frame_halves(
    frame: &[u8],
    top_half_cols_averages: &mut [u8],
    bottom_half_cols_averages: &mut [u8],
) {
    let t = top_half_cols_averages;
    let b = bottom_half_cols_averages;
    debug_assert_ne!(0, frame.len());
    debug_assert_ne!(0, t.len());
    debug_assert_eq!(t.len(), b.len());
    let half = frame.len() / 2;
    debug_assert_eq!(frame.len(), half * 2);
    debug_assert_eq!(0, half % 3);
    debug_assert_eq!(0, half / 3 % t.len());

    // Gets the pixels from the top half of the frame and calculates averages
    // for each column of that half.
    let top_half = &frame[0..half];
    calculate_average_column_gray(top_half, t);

    // Likewise for the bottom half.
    let bottom_half = &frame[half..];
    calculate_average_column_gray(bottom_half, b);
}

fn calculate_average_column_gray(frame: &[u8], averages_store: &mut [u8]) {
    // How many pixels are there in this frame. Each pixel is a chunk of three
    // bytes.
    let col_pixels_count = frame.len() / 3 / averages_store.len();
    // Temporarily stores the averages.
    let mut cols: Vec<_> = (0..averages_store.len()).map(|_| 0.0).collect();
    // Will keep track of the currently iterated column.
    let mut col = 0;

    // Each pixel is represented by 3 bytes, red green and blue.
    for pixels in frame.chunks(3) {
        // Calculates the grayscale from RGB. Preferably we would get the
        // grayscale from the camera input. However this seems HW dependent.
        let grayscale = {
            let r = pixels[0] / 10 * 3;
            let g = pixels[1] / 10 * 6;
            let b = pixels[2] / 10;
            r + g + b
        };

        // Adds the grayscale value of the pixel to the average of the column
        // it belongs to.
        cols[col] += grayscale as f32 / col_pixels_count as f32;

        // Increments col until it reaches end of a row.
        col = if col == cols.len() - 1 { 0 } else { col + 1 };
    }

    // Converts the averages back to a byte for each input.
    for (i, col) in cols.into_iter().enumerate() {
        averages_store[i] = col as u8;
    }
}

fn find_controller(averages: &[u8], frame: &mut [u8]) -> Option<u32> {
    // Calculates the distance from the background and removes mutability.
    distance_from_average(averages, frame);
    let frame: &[u8] = frame;

    // Calculates the average distance from the average playfield.
    let average_distance: u8 = {
        let total = frame.iter().fold(0usize, |sum, col| sum + *col as usize);
        // We can be certain that the average won't overflow a byte.
        (total / frame.len()) as u8
    };

    let rate_span = || -> f32 { 0.0 };

    let mut best_span_rating = 0;
    let mut best_span: Option<(usize, usize)> = None;
    let mut current_span_start: Option<usize> = None;

    for (x, col) in frame.iter().enumerate() {
        let col = *col;
        if col > average_distance && current_span_start.is_none() {
            current_span_start = Some(x);
        }

        if col <= average_distance && current_span_start.is_some() {
            // compare if better
            best_span = Some((current_span_start.unwrap(), x - 1));
        }
    }

    if current_span_start.is_some() {
        // compare if better
    }

    best_span
        .map(|(from, to)| (to - from) / 2 + from)
        .map(|x| x as u32)
}

fn distance_from_average(averages: &[u8], frame: &mut [u8]) {
    debug_assert_eq!(averages.len(), frame.len());
    for (col, average) in frame.iter_mut().zip(averages) {
        let c = *col;
        *col = if c > *average {
            c - average
        } else {
            average - c
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // An image of dimensions 3x14.
    const CAPTURE: &[u8] = &[
        00, 000, 000, /**/ 009, 084, 013, /**/ 120, 230, 103, //
        80, 230, 029, /**/ 000, 000, 000, /**/ 094, 058, 049, //
        39, 239, 123, /**/ 209, 009, 084, /**/ 000, 000, 000, //
        103, 80, 230, /**/ 029, 057, 083, /**/ 028, 094, 058, //
        000, 000, 00, /**/ 199, 240, 178, /**/ 120, 230, 103, //
        80, 230, 029, /**/ 000, 000, 000, /**/ 209, 009, 084, //
        13, 120, 230, /**/ 103, 080, 230, /**/ 209, 129, 093, //
        // -------------------------------------------------------------------- //
        100, 100, 100, /**/ 28, 080, 230, /**/ 029, 057, 083, //
        239, 123, 209, /**/ 100, 100, 100, /**/ 240, 178, 20, //
        230, 103, 029, /**/ 057, 083, 028, /**/ 100, 100, 10, //
        120, 230, 103, /**/ 80, 230, 178, /**/ 120, 230, 103, //
        101, 101, 101, /**/ 94, 084, 013, /**/ 120, 230, 103, //
        080, 230, 029, /**/ 101, 101, 101, /**/ 023, 14, 159, //
        230, 198, 176, /**/ 077, 034, 088, /**/ 109, 73, 022, //
    ];

    #[test]
    fn test_average_gray_for_frame_halves() {
        let mut a = [0; 3];
        let mut b = [0; 3];
        average_gray_for_frame_halves(CAPTURE, &mut a, &mut b);
        assert_eq!(&[98, 68, 100], &a);
        assert_eq!(&[148, 091, 111], &b);
    }

    #[test]
    fn test_distance_from_average() {
        let averages = &[120, 80, 30];
        let mut frame = [110, 90, 30];
        distance_from_average(averages, &mut frame);
        assert_eq!(&[10, 10, 0], &frame);
    }
}
