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
    // MOG2-Warmup: danach wird das Hintergrundmodell eingefroren, damit eine
    // still stehende / duckende Person nicht ins Modell absorbiert wird.
    frames_seen: u32,
    // Szenenwechsel-Erkennung (globaler Lichtwechsel -> Auto-Rekalibrierung)
    scene_high_count: u32,
    pub global_ratio: f64,    // Anteil aktiver Zellen im GESAMTEN Raster
    pub want_recalib: bool,   // true fuer genau einen Frame nach Auto-Rekalib
    // Optical-Flow-Richtungs-Gate fuer die Tap-Zonen
    pub use_flow: bool,
    prev_gray: Mat,           // letztes Graubild in FLOW_WIDTH x FLOW_HEIGHT
    dir_recent: [u32; 3],     // Rest-Frames, in denen die Richtung noch gilt
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
            frames_seen: 0,
            scene_high_count: 0,
            global_ratio: 0.0,
            want_recalib: false,
            use_flow: config::FLOW_ENABLED,
            prev_gray: Mat::default(),
            dir_recent: [0; 3],
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
    /// Setzt auch den Warmup zurueck, damit das frische Modell erst schnell
    /// lernt und dann wieder eingefroren wird.
    pub fn recalibrate(&mut self) {
        if let Ok(bs) = Self::make_subtractor() {
            self.backsub = bs;
        }
        self.frames_seen = 0;
        self.scene_high_count = 0;
    }

    /// Sieht der aktuelle Frame nach globalem Szenenwechsel aus? Solange ja,
    /// sollten keine Events an die Tastatur gehen (Fehl-Tap-Hagel).
    pub fn scene_suspect(&self) -> bool {
        self.global_ratio > config::SCENE_CHANGE_RATIO
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
        self.want_recalib = false;

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

        // MOG2-Maske. Waehrend des Warmups automatisch lernen (-1.0), danach
        // Modell einfrieren - sonst verschwindet eine still gehaltene Pose
        // (Ducken!) nach ~10-25s aus der Maske und der Hold bricht ab.
        let lr = if self.frames_seen < config::MOG2_WARMUP_FRAMES {
            -1.0
        } else {
            config::MOG2_FROZEN_LEARNING_RATE
        };
        self.frames_seen = self.frames_seen.saturating_add(1);
        let mut fg = Mat::default();
        video::BackgroundSubtractorTrait::apply(&mut self.backsub, &self.frame, &mut fg, lr)?;
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
        let mut act_total = 0u32;
        for row in 0..config::GRID_ROWS {
            for col in 0..config::GRID_COLS {
                let v = *small.at_2d::<u8>(row, col)?;
                let on = v as f64 >= config::CELL_ACTIVE_THRESH;
                self.active[(row * config::GRID_COLS + col) as usize] = on;
                if on {
                    act_total += 1;
                }
            }
        }

        // Szenenwechsel-Erkennung: ist fast das ganze Raster laenger aktiv,
        // war das Licht (oder die Kamera), keine Geste -> neu lernen.
        self.global_ratio =
            act_total as f64 / (config::GRID_COLS * config::GRID_ROWS) as f64;
        if self.global_ratio > config::SCENE_CHANGE_RATIO {
            self.scene_high_count += 1;
            if self.scene_high_count >= config::SCENE_CHANGE_FRAMES {
                self.recalibrate(); // setzt scene_high_count zurueck
                self.want_recalib = true;
                println!(">> Szenenwechsel erkannt (Licht?) - Hintergrund wird neu gelernt.");
            }
        } else {
            self.scene_high_count = 0;
        }

        let dir_ok = self.update_flow_gate(&closed)?;
        let zones = self.zone_ratios();
        Ok(Some(ZoneActivity::new(zones, t).with_dir_ok(dir_ok)))
    }

    /// Optical-Flow-Richtungs-Gate fuer die Tap-Zonen [left, right, up].
    ///
    /// Auf einem stark verkleinerten Graubild wird der mittlere Flussvektor
    /// der Vordergrund-Pixel je Zone bestimmt. Nur wenn er in Gestenrichtung
    /// zeigt (left: x negativ, right: x positiv, up: y negativ), oeffnet das
    /// Gate - mit ein paar Frames Gedaechtnis, damit es am Umkehrpunkt der
    /// Bewegung nicht flackert. Ohne Flow-Info (erster Frame, deaktiviert)
    /// bleibt das Gate offen, damit keine Gesten verloren gehen.
    fn update_flow_gate(&mut self, fg_mask: &Mat) -> opencv::Result<[bool; 3]> {
        let mut dir_ok = [true; 3];
        if !self.use_flow {
            return Ok(dir_ok);
        }

        let fw = config::FLOW_WIDTH;
        let fh = config::FLOW_HEIGHT;
        let mut gray = Mat::default();
        imgproc::cvt_color_def(&self.frame, &mut gray, imgproc::COLOR_BGR2GRAY)?;
        let mut gray_small = Mat::default();
        imgproc::resize(&gray, &mut gray_small, Size::new(fw, fh), 0.0, 0.0, imgproc::INTER_AREA)?;

        if !self.prev_gray.empty() {
            let mut flow = Mat::default();
            video::calc_optical_flow_farneback(
                &self.prev_gray, &gray_small, &mut flow,
                0.5, 2, 9, 2, 5, 1.1, 0,
            )?;
            let mut mask_small = Mat::default();
            imgproc::resize(fg_mask, &mut mask_small, Size::new(fw, fh), 0.0, 0.0, imgproc::INTER_NEAREST)?;

            for i in 0..3 {
                let r = &config::ZONE_RECTS[i];
                let x0 = (r.x0 * fw as f64) as i32;
                let x1 = (r.x1 * fw as f64) as i32;
                let y0 = (r.y0 * fh as f64) as i32;
                let y1 = (r.y1 * fh as f64) as i32;
                let rect = core::Rect::new(x0, y0, (x1 - x0).max(1), (y1 - y0).max(1));
                let flow_roi = Mat::roi(&flow, rect)?;
                let mask_roi = Mat::roi(&mask_small, rect)?;

                // Mittelwert nur ueber Vordergrund-Pixel; zu wenige Pixel =
                // keine belastbare Richtung.
                let mut ok_now = false;
                if core::count_non_zero(&mask_roi)? >= config::FLOW_MIN_PIXELS {
                    let m = core::mean(&flow_roi, &mask_roi)?; // [vx, vy, _, _]
                    ok_now = match i {
                        0 => m[0] <= -config::FLOW_MIN_MAG, // left: nach aussen links
                        1 => m[0] >= config::FLOW_MIN_MAG,  // right: nach aussen rechts
                        _ => m[1] <= -config::FLOW_MIN_MAG, // up: aufwaerts
                    };
                }
                if ok_now {
                    self.dir_recent[i] = config::FLOW_MEMORY_FRAMES;
                } else if self.dir_recent[i] > 0 {
                    self.dir_recent[i] -= 1;
                }
                dir_ok[i] = self.dir_recent[i] > 0;
            }
        }
        self.prev_gray = gray_small;
        Ok(dir_ok)
    }
}
