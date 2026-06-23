//! Zentrale Konfiguration (Port von config.py). Reine Konstanten.

use crate::detector::Config as DetectorConfig;
use crate::keysender::Key;

// --- Kamera / Bild ---
pub const CAMERA_INDEX: i32 = 0;
pub const FRAME_WIDTH: i32 = 1920;
pub const FRAME_HEIGHT: i32 = 1080;
pub const MIRROR: bool = true;

// --- Raster ---
pub const GRID_COLS: i32 = 32;
pub const GRID_ROWS: i32 = 24;
pub const CELL_ACTIVE_THRESH: f64 = 40.0; // Maskenwert 0..255

// --- Detector-Schwellen (Anteile 0..1) ---
pub const ZONE_ENTER_RATIO: f64 = 0.15;
pub const ZONE_EXIT_RATIO: f64 = 0.08;
pub const COOLDOWN_S: f64 = 0.5;
pub const CALIB_FRAMES: usize = 30; // ~1s @30FPS

// --- MOG2 ---
pub const MOG2_HISTORY: i32 = 300;
pub const MOG2_VAR_THRESHOLD: f64 = 25.0;
pub const MOG2_DETECT_SHADOWS: bool = true;

// --- KeySender ---
pub const TAP_HOLD_MS: u64 = 40;

// --- Overlay ---
pub const MONITOR2_X_OFFSET: i32 = 1920; // x-Position des Overlay-Fensters (2. Monitor)

/// Relatives Zonen-Rechteck in Bildkoordinaten (0..1).
#[derive(Debug, Clone, Copy)]
pub struct ZoneRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// Zonen in der Detector-Index-Reihenfolge: [left, right, up, down].
/// NICHT ueberlappend; mittleres Feld bleibt neutrale Ruhezone.
pub const ZONE_RECTS: [ZoneRect; 4] = [
    ZoneRect { x0: 0.00, y0: 0.00, x1: 0.34, y1: 0.70 }, // left  -> A
    ZoneRect { x0: 0.66, y0: 0.00, x1: 1.00, y1: 0.70 }, // right -> D
    ZoneRect { x0: 0.34, y0: 0.00, x1: 0.66, y1: 0.38 }, // up    -> Space
    ZoneRect { x0: 0.00, y0: 0.70, x1: 1.00, y1: 1.00 }, // down  -> S (Hold)
];

/// Geste -> Taste.
pub const KEY_LEFT: Key = Key::A;
pub const KEY_RIGHT: Key = Key::D;
pub const KEY_UP: Key = Key::Space;
pub const KEY_DOWN: Key = Key::S;

pub fn detector_config() -> DetectorConfig {
    DetectorConfig {
        calib_frames: CALIB_FRAMES,
        enter_ratio: ZONE_ENTER_RATIO,
        exit_ratio: ZONE_EXIT_RATIO,
        cooldown_s: COOLDOWN_S,
    }
}
