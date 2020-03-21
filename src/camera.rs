use super::WINDOW_SIZE;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const CAMERA_DEV: &str = "/dev/video2";
const FORMAT: &[u8] = b"RGB3";
const MINIMUM_BACKGROUND_DISTANCE: u8 = 10;
const WINDOW_WIDTH: u32 = WINDOW_SIZE.0 as u32;

/// Object used for scanning the camera input. It updates controller positions.
pub struct Camera {
    /// Latest position of each player. This value is updated by the camera.
    /// TODO: Consider making these atomic u32s.
    pub positions: [Arc<Mutex<u32>>; 2],

    // Handle to the started camera which can capture images.
    handle: rscam::Camera,

    // What was the average gray for each column of the top half when the game
    // started. We refer to this initial state as background.
    top_half_bg: [u8; 1280],

    // What was the average gray for each column of the bottom half when the
    // game started. We refer to this initial state as background.
    bottom_half_bg: [u8; 1280],

    // On average, if we capture only the background, consequential frames will
    // differ at most by this amount.
    background_distance_range: u8,
}

impl Camera {
    /// Builds a new empty camera that has be to calibrated.
    pub fn new() -> Self {
        info!(
            "Starting camera in format {}",
            String::from_utf8_lossy(&FORMAT)
        );
        let mut handle =
            rscam::new(CAMERA_DEV).expect("Cannot find camera device");

        for format in handle.formats() {
            let format = format.expect("Cannot read format");
            debug!(
                "{} format is supported: {}.",
                String::from_utf8_lossy(&format.format),
                format.description
            );
        }
        assert!(handle.formats().any(|f| f.unwrap().format == FORMAT));

        // Starts the camera, now we can capture images.
        handle
            .start(&rscam::Config {
                interval: (1, 30),
                resolution: (1280, 720),
                format: FORMAT,
                ..Default::default()
            })
            .expect("Cannot start camera");
        // Tests the camera.
        handle.capture().expect("Cannot take a capture");

        // Some values will be calibrated later.
        Self {
            positions: [
                Arc::new(Mutex::new(WINDOW_WIDTH / 2)),
                Arc::new(Mutex::new(WINDOW_WIDTH / 2)),
            ],
            handle,
            top_half_bg: [0; 1280],
            bottom_half_bg: [0; 1280],
            background_distance_range: MINIMUM_BACKGROUND_DISTANCE,
        }
    }

    /// Captures the empty playfield to learn about its default colours and
    /// inconsistencies. It records this default state and when the camera
    /// thread starts updating the positions, it will calculate them against
    /// this default.
    pub fn map_playfield(&mut self) {
        // Capture the visible field.
        let frame = self.handle.capture().expect("Cannot capture camera input");

        // Sets the initial state of the playfield which we refer to as the
        // background. Each half of the image has its own background.
        average_gray_for_frame_halves(
            &frame,
            &mut self.top_half_bg,
            &mut self.bottom_half_bg,
        );

        let mut top_half = [0; 1280];
        let mut bottom_half = [0; 1280];
        let mut total_distance_over_n_frames = 0usize;
        const FRAMES_TO_TAKE: usize = 3;
        for _ in 0..FRAMES_TO_TAKE {
            let frame =
                self.handle.capture().expect("Cannot capture camera input");
            average_gray_for_frame_halves(
                &frame,
                &mut top_half,
                &mut bottom_half,
            );
            // Calculates the distance from the background and removes mutability.
            distance_from_background(&self.top_half_bg, &mut top_half);
            distance_from_background(&self.bottom_half_bg, &mut bottom_half);

            // Calculates the average distance from the average playfield.
            let avg_top = average_distance(&top_half) as usize;
            let avg_bottom = average_distance(&bottom_half) as usize;
            total_distance_over_n_frames += (avg_top + avg_bottom) / 2;
            thread::sleep(Duration::from_secs(1));
        }

        // We took N frames and each frame had two halves, therefore we need to
        // divide the total by N * 2.
        let average_distance_over_n_frames =
            total_distance_over_n_frames / FRAMES_TO_TAKE;
        self.background_distance_range = (average_distance_over_n_frames as u8)
            .max(MINIMUM_BACKGROUND_DISTANCE);

        debug!(
            "The background is in range of {} shades of grey.",
            self.background_distance_range
        );
    }

    /// Starts a new thread on which the camera continuously screens the
    /// playfield and update the paddle positions.
    pub fn start_capturing(self) -> io::Result<JoinHandle<()>> {
        info!("Starting new thread for camera.");
        let camera_thread = thread::Builder::new().name("camera".to_string());
        camera_thread.spawn(move || {
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
                    self.find_controller(&self.top_half_bg, &mut top_half);
                // let bottom_x =
                // self.ind_controller(&self.bottom_half_bg, &mut bottom_half);

                // Updates the controllers.
                for x in top_x {
                    println!("Updating controller 1 to {}", x);
                    (*self.positions[0].lock().unwrap()) = x;
                }
                // for x in bottom_x {
                //     println!("Updating controller 2 to {}", x);
                //     (*self.positions[1].lock().unwrap()) = x;
                // }
            }
        })
    }
    // Calculates a diff between the "background", that is the state of the
    // playfield when the game started and there were no objects, which we know
    // from the "maps_playfield" phase, and the current frame. If we find
    // sequence of columns which have increased difference, or distance, to the
    // background, we mark those columns as candidates for having the controller
    // positioned there. Returns controller's position relative to the window,
    // not to the width of the frame.
    fn find_controller(
        &self,
        background: &[u8],
        frame: &mut [u8],
    ) -> Option<u32> {
        debug_assert_eq!(background.len(), frame.len());
        // Calculates the distance from the background and removes mutability.
        distance_from_background(background, frame);
        let frame: &[u8] = frame;

        // Calculates the average distance from the average playfield.
        let average_distance = average_distance(frame);

        // If the capture hasn't change all that much, we return early. This
        // limits the size of the controller. Since the game is meant to be
        // played using a hand, it shouldn't be an issue.
        if average_distance < self.background_distance_range {
            return None;
        }
        println!("{}", average_distance);

        // Selects a slice from the array of distances from the background.
        // It sums the squares of those distances.
        let rate_streak = |from: usize, to: usize| -> f32 {
            debug_assert!(to < frame.len());
            debug_assert!(from <= to);
            frame[from..to]
                .iter()
                .fold(0.0, |sum, el| sum + (*el as f32).powi(2))
        };

        // We can default to zero as rating can only be higher.
        let mut best_streak_rating = 0.0;
        // We will store a range which contains the most likely horizontal
        // position of an object which is used to control the paddle.
        let mut best_streak: Option<(usize, usize)> = None;
        // If we find a pattern of distances which are higher than average, we
        // make a note where the increase in distance to the background start.
        let mut candidate_streak: Option<usize> = None;

        for (x, col) in frame.iter().enumerate() {
            let col = *col;
            // If the current column's distance is larger than average distance
            // it will start a new streak as a candidate for the best streak.
            if col > average_distance && candidate_streak.is_none() {
                candidate_streak = Some(x);
                continue;
            }

            // If we found a column which has its average distance lower than
            // average, the streak stops. Note that this also covers the
            // scenario where we arrive to the end of the frame while still on
            // streak.
            if col <= average_distance || x == frame.len() - 1 {
                // If we are currently on a streak, we want to check whether
                // it's more distant from the background than other parts.
                for candidate_start in candidate_streak {
                    // Sums the squares of distances between the frame and the
                    // background.
                    let rating = rate_streak(candidate_start, x - 1);
                    // If this streak was better than previous best, set it as
                    // best.
                    if rating > best_streak_rating {
                        best_streak = Some((candidate_start, x - 1));
                        best_streak_rating = rating;
                    }
                    // Break the streak.
                    candidate_streak = None;
                }
            }
        }

        // Selects the mid of the streak. That means if the streak is 30 pixels
        // long and it starts at x = 30, then we return 45.
        best_streak
            .map(|(from, to)| (to - from) / 2 + from)
            // Calculates the position of the controller relative to the window
            // width. If the camera input is 1280px and the window is 500 px
            // wide then a controller at 640px of camera input should be
            // positioned to 250px of window.
            .map(|x| x * WINDOW_WIDTH as usize / frame.len())
            .map(|x| x as u32)
    }
}

// From input frame converts all pixels to grayscale. Then it horizontally
// splits the image into halves. For each half it calculates the average grey at
// each column.
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

// Converts RGB pixels to grayscale and then calculates an average for each
// column.
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

// Calculates the average distance from the background playfield.
fn average_distance(frame: &[u8]) -> u8 {
    let total = frame.iter().fold(0usize, |sum, col| sum + *col as usize);
    // We can be certain that the average won't overflow a byte.
    (total / frame.len()) as u8
}

// Changes the value of each column to the value of its distance to the
// background frame.
fn distance_from_background(background: &[u8], frame: &mut [u8]) {
    debug_assert_eq!(background.len(), frame.len());
    for (col, bg_average) in frame.iter_mut().zip(background) {
        let c = *col;
        *col = if c > *bg_average {
            c - bg_average
        } else {
            bg_average - c
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
        // -------------------------------------------------- //
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
    fn test_distance_from_background() {
        let averages = &[120, 80, 30];
        let mut frame = [110, 90, 30];
        distance_from_background(averages, &mut frame);
        assert_eq!(&[10, 10, 0], &frame);
    }
}
