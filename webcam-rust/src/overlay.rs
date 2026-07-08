//! Beweis-Screen / Overlay auf Monitor 2 (Port von overlay.py).
//!
//! Gezeichnet wird mit OpenCV-imgproc auf den Frame; Fenster + Tasten laufen
//! ueber minifb (reines Rust, kein Qt/highgui). Tasten: 'm' Bild/Maske,
//! 'c' neu kalibrieren, 'k' Send an/aus, 'q'/ESC Ende.

use std::time::Instant;

use minifb::{Key as MKey, KeyRepeat, Window, WindowOptions};
use opencv::core::{self, Mat, Point, Scalar, Size};
use opencv::prelude::*;
use opencv::imgproc;

use crate::config;
use crate::keysender::Key;

const FONT: i32 = imgproc::FONT_HERSHEY_SIMPLEX;
const AA: i32 = imgproc::LINE_AA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Recalibrate,
    ToggleSend,
}

pub struct Overlay {
    window: Window,
    buf: Vec<u32>,
    show_mask: bool,
    last_t: Option<Instant>,
    fps: f64,
}

impl Overlay {
    pub fn new() -> Result<Self, minifb::Error> {
        let w = config::FRAME_WIDTH as usize;
        let h = config::FRAME_HEIGHT as usize;
        let mut window = Window::new(
            "Webcam-Steuerung (Beweis-Screen)",
            w,
            h,
            WindowOptions { resize: true, ..WindowOptions::default() },
        )?;
        window.set_position(config::MONITOR2_X_OFFSET as isize, 0);
        // ~60 FPS Begrenzung der Fenster-Updates
        window.set_target_fps(60);
        Ok(Self {
            window,
            buf: vec![0; w * h],
            show_mask: false,
            last_t: None,
            fps: 0.0,
        })
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    /// Fenster wird beim Drop geschlossen; hier nur zur API-Symmetrie.
    pub fn close(&self) {}

    fn base_image(&self, frame: &Mat, active: &[bool]) -> opencv::Result<Mat> {
        if self.show_mask {
            let rows = config::GRID_ROWS;
            let cols = config::GRID_COLS;
            let mut small =
                Mat::new_rows_cols_with_default(rows, cols, core::CV_8UC1, Scalar::all(0.0))?;
            for row in 0..rows {
                for col in 0..cols {
                    if active[(row * cols + col) as usize] {
                        *small.at_2d_mut::<u8>(row, col)? = 255;
                    }
                }
            }
            let mut big = Mat::default();
            imgproc::resize(&small, &mut big, Size::new(frame.cols(), frame.rows()),
                0.0, 0.0, imgproc::INTER_NEAREST)?;
            let mut bgr = Mat::default();
            imgproc::cvt_color_def(&big, &mut bgr, imgproc::COLOR_GRAY2BGR)?;
            Ok(bgr)
        } else {
            frame.try_clone()
        }
    }

    /// Zeichnet alles auf den Frame und gibt das fertige BGR-Bild zurueck.
    fn render(
        &mut self,
        frame: &Mat,
        active: &[bool],
        calibrated: bool,
        current_key: Option<Key>,
        triggered_zone: Option<usize>,
        send_active: bool,
    ) -> opencv::Result<Mat> {
        let mut img = self.base_image(frame, active)?;
        let w = img.cols();
        let h = img.rows();
        let cols = config::GRID_COLS;
        let rows = config::GRID_ROWS;

        // aktive Zellen halbtransparent
        let mut layer = img.try_clone()?;
        let cw = w as f64 / cols as f64;
        let ch = h as f64 / rows as f64;
        for row in 0..rows {
            for col in 0..cols {
                if active[(row * cols + col) as usize] {
                    let p0 = Point::new((col as f64 * cw) as i32, (row as f64 * ch) as i32);
                    let p1 = Point::new(((col + 1) as f64 * cw) as i32, ((row + 1) as f64 * ch) as i32);
                    imgproc::rectangle_points(&mut layer, p0, p1, Scalar::new(0.0, 200.0, 255.0, 0.0), -1, AA, 0)?;
                }
            }
        }
        let mut blended = Mat::default();
        core::add_weighted(&layer, 0.35, &img, 0.65, 0.0, &mut blended, -1)?;
        img = blended;

        // Rasterlinien
        let grid_col = Scalar::new(60.0, 60.0, 60.0, 0.0);
        for c in 1..cols {
            let x = c * w / cols;
            imgproc::line(&mut img, Point::new(x, 0), Point::new(x, h), grid_col, 1, AA, 0)?;
        }
        for r in 1..rows {
            let y = r * h / rows;
            imgproc::line(&mut img, Point::new(0, y), Point::new(w, y), grid_col, 1, AA, 0)?;
        }

        // Zonenrahmen + getriggerte Zone
        let names = ["left", "right", "up", "down"];
        for (i, rect) in config::ZONE_RECTS.iter().enumerate() {
            let p0 = Point::new((rect.x0 * w as f64) as i32, (rect.y0 * h as f64) as i32);
            let p1 = Point::new((rect.x1 * w as f64) as i32, (rect.y1 * h as f64) as i32);
            let triggered = triggered_zone == Some(i);
            let color = if triggered { Scalar::new(0.0, 0.0, 255.0, 0.0) } else { Scalar::new(0.0, 255.0, 0.0, 0.0) };
            let thick = if triggered { 4 } else { 1 };
            imgproc::rectangle_points(&mut img, p0, p1, color, thick, AA, 0)?;
            imgproc::put_text(&mut img, names[i], Point::new(p0.x + 4, p0.y + 18), FONT, 0.5, color, 1, AA, false)?;
        }

        // grosse Tastenanzeige
        if let Some(k) = current_key {
            let label = match k {
                Key::A => "<- A",
                Key::D => "D ->",
                Key::Space => "[ SPRUNG ]",
                Key::S => "v S",
            };
            imgproc::put_text(&mut img, label, Point::new((w as f64 * 0.30) as i32, 50), FONT, 1.4,
                Scalar::new(0.0, 0.0, 255.0, 0.0), 3, AA, false)?;
        }

        // SEND-Status oben rechts
        let (txt, col) = if send_active {
            ("SEND: AN", Scalar::new(0.0, 0.0, 255.0, 0.0))
        } else {
            ("SEND: AUS", Scalar::new(180.0, 180.0, 180.0, 0.0))
        };
        let mut baseline = 0;
        let ts = imgproc::get_text_size(txt, FONT, 0.9, 2, &mut baseline)?;
        imgproc::put_text(&mut img, txt, Point::new(w - ts.width - 12, 36), FONT, 0.9, col, 2, AA, false)?;

        // FPS
        let now = Instant::now();
        if let Some(prev) = self.last_t {
            let dt = now.duration_since(prev).as_secs_f64();
            if dt > 0.0 {
                self.fps = 0.9 * self.fps + 0.1 * (1.0 / dt);
            }
        }
        self.last_t = Some(now);

        let e = config::ZONE_ENTER_RATIOS;
        let x = config::ZONE_EXIT_RATIOS;
        let status = format!(
            "FPS {:4.1} | Grid {}x{} | enter L{:.2} R{:.2} U{:.2} D{:.2} exit L{:.2} R{:.2} U{:.2} D{:.2} | {} | m=Maske c=Kalib k=Send q=Ende",
            self.fps, cols, rows, e[0], e[1], e[2], e[3], x[0], x[1], x[2], x[3],
            if calibrated { "KALIBRIERT" } else { "kalibriere..." }
        );
        imgproc::put_text(&mut img, &status, Point::new(6, h - 8), FONT, 0.45,
            Scalar::new(255.0, 255.0, 255.0, 0.0), 1, AA, false)?;

        Ok(img)
    }

    /// BGR-Mat -> u32-ARGB-Puffer fuer minifb.
    fn blit(&mut self, img: &Mat) -> opencv::Result<(usize, usize)> {
        let w = img.cols() as usize;
        let h = img.rows() as usize;
        if self.buf.len() != w * h {
            self.buf.resize(w * h, 0);
        }
        let bytes = img.data_bytes()?; // BGR, row-major (Mat ist continuous)
        for i in 0..(w * h) {
            let b = bytes[3 * i] as u32;
            let g = bytes[3 * i + 1] as u32;
            let r = bytes[3 * i + 2] as u32;
            self.buf[i] = (r << 16) | (g << 8) | b;
        }
        Ok((w, h))
    }

    pub fn show(
        &mut self,
        frame: &Mat,
        active: &[bool],
        calibrated: bool,
        current_key: Option<Key>,
        triggered_zone: Option<usize>,
        send_active: bool,
    ) -> opencv::Result<Action> {
        if !frame.empty() {
            let img = self.render(frame, active, calibrated, current_key, triggered_zone, send_active)?;
            let (w, h) = self.blit(&img)?;
            let _ = self.window.update_with_buffer(&self.buf, w, h);
        } else {
            self.window.update();
        }

        if !self.window.is_open() || self.window.is_key_down(MKey::Escape) || self.window.is_key_down(MKey::Q) {
            return Ok(Action::Quit);
        }
        for key in self.window.get_keys_pressed(KeyRepeat::No) {
            match key {
                MKey::M => self.show_mask = !self.show_mask,
                MKey::C => return Ok(Action::Recalibrate),
                MKey::K => return Ok(Action::ToggleSend),
                _ => {}
            }
        }
        Ok(Action::None)
    }
}
