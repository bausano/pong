use super::SCREEN_SIZE;
use std::sync::{Arc, Mutex};

const CAMERA_DEV: &str = "/dev/video2";
const RGB: &[u8] = b"RGB3";

/// Object used for scanning the camera input. It is updated with relevant values such as
/// controller patterns (e.g. a finger) or the controller position.
pub struct Camera {
    /// Latest position of each player. This value is updated by the camera.
    /// TODO: Consider making these atomic u32s.
    pub positions: [Arc<Mutex<f32>>; 2],

    /// Handle to the started camera which can capture images.
    handle: rscam::Camera,

    /// What was the average gray for each column of the top half when the game
    /// started.
    top_half_averages: [u8; 1280],

    /// What was the average gray for each column of the bottom half when the
    /// game started.
    bottom_half_averages: [u8; 1280],
}

impl Camera {
    /// Builds a new empty camera that has be to calibrated.
    pub fn new() -> Self {
        let mut handle = rscam::new(CAMERA_DEV).expect("Cannot find camera device");

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
                Arc::new(Mutex::new(SCREEN_SIZE.1 / 2.0)),
                Arc::new(Mutex::new(SCREEN_SIZE.1 / 2.0)),
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

#[cfg(test)]
mod tests {
    use super::*;

    // An image of dimensions 4x10.
    const CAPTURE: &[u8] = &[
        000, 000, 000, /**/ 009, 084, 013, /**/ 120, 230, 103, /**/ 080, 230, 029,
        000, 000, 000, /**/ 094, 058, 049, /**/ 039, 239, 123, /**/ 209, 009, 084,
        000, 000, 000, /**/ 103, 080, 230, /**/ 029, 057, 083, /**/ 028, 094, 058,
        000, 000, 000, /**/ 199, 240, 178, /**/ 120, 230, 103, /**/ 080, 230, 029,
        000, 000, 000, /**/ 209, 009, 084, /**/ 013, 120, 230, /**/ 103, 080, 230,
        // -------------------------------------------------------------------- //
        100, 100, 100, /**/ 028, 080, 230, /**/ 029, 057, 083, /**/ 239, 123, 209,
        100, 100, 100, /**/ 240, 178, 120, /**/ 230, 103, 029, /**/ 057, 083, 028,
        100, 100, 100, /**/ 120, 230, 103, /**/ 080, 230, 178, /**/ 120, 230, 103,
        101, 101, 101, /**/ 094, 084, 013, /**/ 120, 230, 103, /**/ 080, 230, 029,
        101, 101, 101, /**/ 023, 014, 159, /**/ 230, 198, 176, /**/ 077, 034, 088,
    ];

    #[test]
    fn test_average_gray_for_frame_halves() {
        let mut a = [0; 4];
        let mut b = [0; 4];
        average_gray_for_frame_halves(CAPTURE, &mut a, &mut b);
        assert_eq!(&[000, 099, 133, 112], &a);
        assert_eq!(&[100, 109, 147, 124], &b);
    }
}
