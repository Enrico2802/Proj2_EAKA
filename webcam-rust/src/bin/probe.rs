//! Bau-Beweis: belegt, dass die opencv-Crate baut, linkt UND zur Laufzeit
//! eine Kamera oeffnen kann. Liest ein paar Frames und gibt die Bildgroesse aus.
//!
//! Ausfuehren:  cargo run --bin probe

use opencv::{prelude::*, videoio, core};

fn main() -> opencv::Result<()> {
    println!("OpenCV-Version: {}", core::CV_VERSION);

    // Kamera 0 oeffnen (DSHOW-Backend wie im Python-Port)
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
