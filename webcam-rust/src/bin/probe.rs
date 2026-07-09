//! Build proof: verifies that the opencv crate builds, links AND can open a
//! camera at runtime. Reads a few frames and prints the frame size.
//!
//! Run with:  cargo run --bin probe

use opencv::{prelude::*, videoio, core};

fn main() -> opencv::Result<()> {
    println!("OpenCV-Version: {}", core::CV_VERSION);

    // DSHOW backend, matching the main webcam source
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_DSHOW)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        eprintln!("Kamera 0 konnte nicht geoeffnet werden.");
        std::process::exit(1);
    }

    let mut frame = core::Mat::default();
    for i in 0..5 {
        cam.read(&mut frame)?;
        if frame.empty() {
            println!("Frame {i}: leer");
            continue;
        }
        println!("Frame {i}: {}x{}", frame.cols(), frame.rows());
    }

    println!("OK: opencv-Crate baut, linkt und liest Kamera-Frames.");
    Ok(())
}
