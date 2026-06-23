//! Echte Quelle: Webcam -> MOG2-Maske -> Raster -> Zonen-Anteile (Port von
//! webcam_source.py). Haelt zusaetzlich das letzte Bild + das aktive Raster
//! fuer das Overlay vor.

use std::time::Instant;

use opencv::core::{self, Mat, Point, Size};
use opencv::prelude::*;
use opencv::{imgproc, videoio};
use opencv::video::{self, BackgroundSubtractorMOG2};
use opencv::core::Ptr;

use crate::config;
use crate::zones::{ZoneActivity, Zones};

pub struct WebcamGridSource {
    cap: videoio::VideoCapture,
    backsub: Ptr<BackgroundSubtractorMOG2>,
    kernel: Mat,
    mirror: bool,
    t0: Option<Instant>,
    // fuer das Overlay:
    pub frame: Mat,           // letztes (ggf. gespiegeltes) BGR-Bild
    pub active: Vec<bool>,    // aktives Raster, ROWS*COLS (row-major)
}

impl WebcamGridSource {
    pub fn new(camera_index: i32, mirror: bool) -> opencv::Result<Self> {
        let mut cap = videoio::VideoCapture::new(camera_index, videoio::CAP_DSHOW)?;
        if !cap.is_opened()? {
            cap = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
        }
        if !cap.is_opened()? {
            return Err(opencv::Error::new(
                core::StsError,
                format!("Kamera {camera_index} konnte nicht geoeffnet werden."),
            ));
        }
        // MJPG erzwingen (vor UND nach der Aufloesung), sonst YUY2 -> ~5 FPS.
        let mjpg = videoio::VideoWriter::fourcc('M', 'J', 'P', 'G')?;
        cap.set(videoio::CAP_PROP_FOURCC, mjpg as f64)?;
        cap.set(videoio::CAP_PROP_FRAME_WIDTH, config::FRAME_WIDTH as f64)?;
        cap.set(videoio::CAP_PROP_FRAME_HEIGHT, config::FRAME_HEIGHT as f64)?;
        cap.set(videoio::CAP_PROP_FOURCC, mjpg as f64)?;
        let aw = cap.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
        let ah = cap.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;
        println!("Kamera {camera_index}: {aw}x{ah}");

        let backsub = Self::make_subtractor()?;
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?;

        Ok(Self {
            cap,
            backsub,
            kernel,
            mirror,
            t0: None,
            frame: Mat::default(),
            active: vec![false; (config::GRID_COLS * config::GRID_ROWS) as usize],
        })
    }

    fn make_subtractor() -> opencv::Result<Ptr<BackgroundSubtractorMOG2>> {
        video::create_background_subtractor_mog2(
            config::MOG2_HISTORY,
            config::MOG2_VAR_THRESHOLD,
            config::MOG2_DETECT_SHADOWS,
        )
    }

    /// Hintergrundmodell neu lernen (Licht/Standort hat sich geaendert).
    pub fn recalibrate(&mut self) {
        if let Ok(bs) = Self::make_subtractor() {
            self.backsub = bs;
        }
    }

    fn zone_ratios(&self) -> Zones {
        let cols = config::GRID_COLS;
        let rows = config::GRID_ROWS;
        let mut out = [0.0f64; 4];
        for (i, r) in config::ZONE_RECTS.iter().enumerate() {
            let c0 = (r.x0 * cols as f64) as i32;
            let c1 = (r.x1 * cols as f64) as i32;
            let r0 = (r.y0 * rows as f64) as i32;
            let r1 = (r.y1 * rows as f64) as i32;
            let mut total = 0u32;
            let mut act = 0u32;
            for row in r0..r1 {
                for col in c0..c1 {
                    total += 1;
                    if self.active[(row * cols + col) as usize] {
                        act += 1;
                    }
                }
            }
            out[i] = if total > 0 { act as f64 / total as f64 } else { 0.0 };
        }
        Zones::new(out[0], out[1], out[2], out[3])
    }

    /// Naechster Frame. Gibt None nur bei echtem Kamera-Ende zurueck.
    pub fn next(&mut self) -> opencv::Result<Option<ZoneActivity>> {
        let mut raw = Mat::default();
        if !self.cap.read(&mut raw)? || raw.empty() {
            return Ok(None);
        }
        if self.t0.is_none() {
            self.t0 = Some(Instant::now());
        }
        let t = self.t0.unwrap().elapsed().as_secs_f64();

        // spiegeln (Selfie-Ansicht)
        if self.mirror {
            let mut flipped = Mat::default();
            core::flip(&raw, &mut flipped, 1)?;
            self.frame = flipped;
        } else {
            self.frame = raw;
        }

        // MOG2-Maske
        let mut fg = Mat::default();
        video::BackgroundSubtractorTrait::apply(&mut self.backsub, &self.frame, &mut fg, -1.0)?;
        // Schatten (127) entfernen -> harter Vordergrund
        let mut bin = Mat::default();
        imgproc::threshold(&fg, &mut bin, 200.0, 255.0, imgproc::THRESH_BINARY)?;
        // Morphologie: open dann close
        let mut opened = Mat::default();
        imgproc::morphology_ex_def(&bin, &mut opened, imgproc::MORPH_OPEN, &self.kernel)?;
        let mut closed = Mat::default();
        imgproc::morphology_ex_def(&opened, &mut closed, imgproc::MORPH_CLOSE, &self.kernel)?;
        // auf Raster skalieren
        let mut small = Mat::default();
        imgproc::resize(
            &closed,
            &mut small,
            Size::new(config::GRID_COLS, config::GRID_ROWS),
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;

        // aktive Zellen
        for row in 0..config::GRID_ROWS {
            for col in 0..config::GRID_COLS {
                let v = *small.at_2d::<u8>(row, col)?;
                self.active[(row * config::GRID_COLS + col) as usize] =
                    v as f64 >= config::CELL_ACTIVE_THRESH;
            }
        }

        let zones = self.zone_ratios();
        Ok(Some(ZoneActivity::new(zones, t)))
    }
}
