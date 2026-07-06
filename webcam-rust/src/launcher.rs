//! Start-GUI vor dem Beweis-Screen (minifb + OpenCV-Zeichnen, kein Qt/highgui).
//!
//! Listet verfuegbare Kameras (Index-Probe via DirectShow) zur Quellenauswahl
//! und bietet drei Aktionen: "Real testen" (echte Tasteneingaben), "Demo"
//! (nur Erkennung, Dry-Run) und "Abbrechen" (Programm beenden).

use minifb::{Key as MKey, MouseButton, MouseMode, Window, WindowOptions};
use opencv::core::{Mat, Point, Scalar, Size};
use opencv::prelude::*;
use opencv::imgproc;
use opencv::videoio;

const FONT: i32 = imgproc::FONT_HERSHEY_SIMPLEX;
const AA: i32 = imgproc::LINE_AA;
const W: i32 = 500;
const H: i32 = 420;
/// Supersampling: intern wird SS-fach groesser gezeichnet und am Ende mit
/// INTER_AREA heruntergefiltert -> glatte Schrift/Kanten statt pixeliger
/// 1-Pixel-Hershey-Strokes.
const SS: i32 = 3;
/// Hoechster Kamera-Index, der beim Suchen probiert wird.
const PROBE_MAX_INDEX: i32 = 4;

#[derive(Debug, Clone, Copy)]
pub struct CameraInfo {
    pub index: i32,
    pub width: i32,
    pub height: i32,
}

/// Ergebnis der Start-GUI: gewaehlte Kamera + ob echte Tasten gesendet werden.
#[derive(Debug, Clone, Copy)]
pub struct LaunchConfig {
    pub camera: i32,
    pub send: bool,
}

/// Probiert Kamera-Indizes 0..=PROBE_MAX_INDEX und liefert die gefundenen
/// Kameras mit Standard-Aufloesung. Erst DirectShow, dann CAP_ANY --
/// dieselbe Fallback-Reihenfolge wie WebcamGridSource (die MSYS2-OpenCV-Build
/// kann DSHOW teils nicht per Index oeffnen).
pub fn probe_cameras() -> Vec<CameraInfo> {
    let mut found = Vec::new();
    for i in 0..=PROBE_MAX_INDEX {
        let mut cap = videoio::VideoCapture::new(i, videoio::CAP_DSHOW).ok();
        if !cap.as_mut().map_or(false, |c| c.is_opened().unwrap_or(false)) {
            cap = videoio::VideoCapture::new(i, videoio::CAP_ANY).ok();
        }
        let Some(mut cap) = cap else { continue };
        if cap.is_opened().unwrap_or(false) {
            let w = cap.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap_or(0.0) as i32;
            let h = cap.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap_or(0.0) as i32;
            println!("  gefunden: Kamera {i} ({w}x{h})");
            found.push(CameraInfo { index: i, width: w, height: h });
        }
        let _ = cap.release();
    }
    found
}

#[derive(Debug, Clone, Copy)]
struct Btn {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Btn {
    fn hit(&self, mx: f32, my: f32) -> bool {
        mx >= self.x as f32
            && mx < (self.x + self.w) as f32
            && my >= self.y as f32
            && my < (self.y + self.h) as f32
    }
}

const BTN_RESCAN: Btn = Btn { x: W - 160, y: 58, w: 140, h: 30 };
const BTN_Y: i32 = H - 78;
const BTN_REAL: Btn = Btn { x: 20, y: BTN_Y, w: 140, h: 48 };
const BTN_DEMO: Btn = Btn { x: 180, y: BTN_Y, w: 140, h: 48 };
const BTN_CANCEL: Btn = Btn { x: 340, y: BTN_Y, w: 140, h: 48 };

fn camera_row(i: usize) -> Btn {
    Btn { x: 20, y: 104 + i as i32 * 42, w: W - 40, h: 34 }
}

// Alle Zeichen-Helfer arbeiten in logischen Koordinaten (W x H) und skalieren
// intern auf die SS-fache Zeichenflaeche.

fn fill_rect(img: &mut Mat, r: Btn, color: Scalar, thickness: i32) -> opencv::Result<()> {
    imgproc::rectangle_points(
        img,
        Point::new(r.x * SS, r.y * SS),
        Point::new((r.x + r.w) * SS, (r.y + r.h) * SS),
        color,
        if thickness > 0 { thickness * SS } else { thickness },
        AA,
        0,
    )
}

fn put(
    img: &mut Mat,
    txt: &str,
    x: i32,
    y: i32,
    scale: f64,
    color: Scalar,
    thick: i32,
) -> opencv::Result<()> {
    imgproc::put_text(
        img,
        txt,
        Point::new(x * SS, y * SS),
        FONT,
        scale * SS as f64,
        color,
        thick * SS,
        AA,
        false,
    )
}

fn text_centered(
    img: &mut Mat,
    txt: &str,
    r: Btn,
    scale: f64,
    color: Scalar,
    thick: i32,
) -> opencv::Result<()> {
    let mut baseline = 0;
    let ts = imgproc::get_text_size(txt, FONT, scale * SS as f64, thick * SS, &mut baseline)?;
    let org = Point::new(
        r.x * SS + (r.w * SS - ts.width) / 2,
        r.y * SS + (r.h * SS + ts.height) / 2,
    );
    imgproc::put_text(img, txt, org, FONT, scale * SS as f64, color, thick * SS, AA, false)
}

fn draw_button(
    img: &mut Mat,
    r: Btn,
    label: &str,
    fill: Scalar,
    enabled: bool,
    hover: bool,
) -> opencv::Result<()> {
    let (fill, text_col) = if !enabled {
        (Scalar::new(55.0, 55.0, 55.0, 0.0), Scalar::new(120.0, 120.0, 120.0, 0.0))
    } else if hover {
        (
            Scalar::new(fill[0] + 35.0, fill[1] + 35.0, fill[2] + 35.0, 0.0),
            Scalar::new(255.0, 255.0, 255.0, 0.0),
        )
    } else {
        (fill, Scalar::new(255.0, 255.0, 255.0, 0.0))
    };
    fill_rect(img, r, fill, -1)?;
    fill_rect(img, r, Scalar::new(110.0, 110.0, 110.0, 0.0), 1)?;
    text_centered(img, label, r, 0.6, text_col, 2)
}

fn render(
    cams: &[CameraInfo],
    selected: usize,
    mouse: Option<(f32, f32)>,
) -> opencv::Result<Mat> {
    let mut img = Mat::new_rows_cols_with_default(
        H * SS,
        W * SS,
        opencv::core::CV_8UC3,
        Scalar::new(35.0, 35.0, 35.0, 0.0),
    )?;
    let (mx, my) = mouse.unwrap_or((-1.0, -1.0));
    let white = Scalar::new(255.0, 255.0, 255.0, 0.0);
    let gray = Scalar::new(200.0, 200.0, 200.0, 0.0);

    put(&mut img, "Webcam-Steuerung", 20, 38, 0.9, white, 2)?;
    put(&mut img, "Quelle waehlen:", 20, 84, 0.6, gray, 1)?;
    draw_button(
        &mut img,
        BTN_RESCAN,
        "Neu suchen",
        Scalar::new(70.0, 70.0, 70.0, 0.0),
        true,
        BTN_RESCAN.hit(mx, my),
    )?;

    if cams.is_empty() {
        put(&mut img, "Keine Kamera gefunden.", 24, 128, 0.6, Scalar::new(0.0, 0.0, 220.0, 0.0), 2)?;
    }
    for (i, cam) in cams.iter().enumerate() {
        let r = camera_row(i);
        let is_sel = i == selected;
        let fill = if is_sel {
            Scalar::new(90.0, 60.0, 20.0, 0.0)
        } else if r.hit(mx, my) {
            Scalar::new(65.0, 65.0, 65.0, 0.0)
        } else {
            Scalar::new(50.0, 50.0, 50.0, 0.0)
        };
        fill_rect(&mut img, r, fill, -1)?;
        let border = if is_sel {
            Scalar::new(255.0, 180.0, 80.0, 0.0)
        } else {
            Scalar::new(100.0, 100.0, 100.0, 0.0)
        };
        fill_rect(&mut img, r, border, if is_sel { 2 } else { 1 })?;
        let marker = if is_sel { "(x)" } else { "( )" };
        let label = format!("{marker} Kamera {} - {}x{}", cam.index, cam.width, cam.height);
        put(&mut img, &label, r.x + 10, r.y + 23, 0.55, white, 1)?;
    }

    let hint1 = "Real testen: Gesten senden echte Tasteneingaben (A/D/S/Leertaste).";
    let hint2 = "Demo: nur Erkennung + Overlay, keine Tasten. Esc = Abbrechen.";
    put(&mut img, hint1, 20, BTN_Y - 32, 0.42, gray, 1)?;
    put(&mut img, hint2, 20, BTN_Y - 12, 0.42, gray, 1)?;

    let has_cam = !cams.is_empty();
    draw_button(&mut img, BTN_REAL, "Real testen", Scalar::new(0.0, 120.0, 0.0, 0.0), has_cam, BTN_REAL.hit(mx, my))?;
    draw_button(&mut img, BTN_DEMO, "Demo", Scalar::new(150.0, 90.0, 20.0, 0.0), has_cam, BTN_DEMO.hit(mx, my))?;
    draw_button(&mut img, BTN_CANCEL, "Abbrechen", Scalar::new(50.0, 50.0, 140.0, 0.0), true, BTN_CANCEL.hit(mx, my))?;

    // Supersampling aufloesen: glatt auf Fenstergroesse herunterfiltern.
    let mut out = Mat::default();
    imgproc::resize(&img, &mut out, Size::new(W, H), 0.0, 0.0, imgproc::INTER_AREA)?;
    Ok(out)
}

/// BGR-Mat -> u32-ARGB-Puffer fuer minifb (wie Overlay::blit).
fn blit(img: &Mat, buf: &mut Vec<u32>) -> opencv::Result<()> {
    let n = (img.cols() * img.rows()) as usize;
    if buf.len() != n {
        buf.resize(n, 0);
    }
    let bytes = img.data_bytes()?;
    for i in 0..n {
        let b = bytes[3 * i] as u32;
        let g = bytes[3 * i + 1] as u32;
        let r = bytes[3 * i + 2] as u32;
        buf[i] = (r << 16) | (g << 8) | b;
    }
    Ok(())
}

/// Zeigt die Start-GUI. `preselect` ist der bevorzugte Kamera-Index (CLI).
/// Rueckgabe: Some(config) bei "Real testen"/"Demo", None bei Abbruch.
pub fn run_launcher(preselect: i32) -> Result<Option<LaunchConfig>, String> {
    println!("Suche Kameras...");
    let mut cams = probe_cameras();
    let mut selected = cams
        .iter()
        .position(|c| c.index == preselect)
        .unwrap_or(0);

    let mut window = Window::new(
        "Webcam-Steuerung - Start",
        W as usize,
        H as usize,
        WindowOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    window.set_target_fps(60);

    let mut buf: Vec<u32> = Vec::new();
    let mut prev_down = false;

    while window.is_open() {
        if window.is_key_down(MKey::Escape) {
            return Ok(None);
        }
        // Enter = sicherer Standard (Demo, keine echten Tasten)
        if window.is_key_down(MKey::Enter) && !cams.is_empty() {
            return Ok(Some(LaunchConfig { camera: cams[selected].index, send: false }));
        }

        let mouse = window.get_mouse_pos(MouseMode::Discard);
        let down = window.get_mouse_down(MouseButton::Left);
        let clicked = down && !prev_down;
        prev_down = down;

        if clicked {
            if let Some((mx, my)) = mouse {
                if BTN_RESCAN.hit(mx, my) {
                    println!("Suche Kameras...");
                    cams = probe_cameras();
                    if selected >= cams.len() {
                        selected = 0;
                    }
                }
                for i in 0..cams.len() {
                    if camera_row(i).hit(mx, my) {
                        selected = i;
                    }
                }
                if !cams.is_empty() && BTN_REAL.hit(mx, my) {
                    return Ok(Some(LaunchConfig { camera: cams[selected].index, send: true }));
                }
                if !cams.is_empty() && BTN_DEMO.hit(mx, my) {
                    return Ok(Some(LaunchConfig { camera: cams[selected].index, send: false }));
                }
                if BTN_CANCEL.hit(mx, my) {
                    return Ok(None);
                }
            }
        }

        let img = render(&cams, selected, mouse).map_err(|e| e.to_string())?;
        blit(&img, &mut buf).map_err(|e| e.to_string())?;
        window
            .update_with_buffer(&buf, W as usize, H as usize)
            .map_err(|e| e.to_string())?;
    }
    Ok(None) // Fenster geschlossen
}
