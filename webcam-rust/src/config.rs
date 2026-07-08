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
// Pro Zone [left, right, up, down]: die Gesten fuellen ihre Zonen sehr
// unterschiedlich (duenner Arm in der kleinen up-Zone vs. ganzer Koerper im
// down-Band), eine gemeinsame Schwelle passt nie fuer alle.
// down-exit bewusst niedrig: waehrend eines langen Holds verblasst die
// MOG2-Maske allmaehlich, ein hoher exit wuerde den Hold verfrueht beenden.
pub const ZONE_ENTER_RATIOS: [f64; 4] = [0.15, 0.15, 0.10, 0.22];
pub const ZONE_EXIT_RATIOS: [f64; 4] = [0.08, 0.08, 0.05, 0.08];
pub const COOLDOWN_S: f64 = 0.5;
pub const CALIB_FRAMES: usize = 30; // ~1s @30FPS
// Flanke erst nach N Frames in Folge ueber/unter der Schwelle (Debounce
// gegen Ein-Frame-Ausreisser; 1 = altes Verhalten, sofort).
pub const CONFIRM_FRAMES: u32 = 2;

// --- MOG2 ---
pub const MOG2_HISTORY: i32 = 300;
pub const MOG2_VAR_THRESHOLD: f64 = 25.0;
pub const MOG2_DETECT_SHADOWS: bool = true;
// Nach dem Warmup wird das Hintergrundmodell eingefroren (Lernrate 0.0),
// sonst wird eine still stehende / duckende Person in ~10-25s ins Modell
// absorbiert und ein gehaltenes S bricht mit falschem HoldDownEnd ab.
// Lichtwechsel fangen Auto-Rekalibrierung (unten) und Taste 'c' ab.
pub const MOG2_WARMUP_FRAMES: u32 = 60; // ~2s @30FPS, deckt CALIB_FRAMES ab
pub const MOG2_FROZEN_LEARNING_RATE: f64 = 0.0;

// --- Auto-Rekalibrierung bei Szenenwechsel (Licht an/aus, Kamera bewegt) ---
// Ist ein Grossteil ALLER Rasterzellen ueber laengere Zeit aktiv, ist das
// keine Geste, sondern ein globaler Beleuchtungswechsel.
pub const SCENE_CHANGE_RATIO: f64 = 0.6;
pub const SCENE_CHANGE_FRAMES: u32 = 45; // ~1.5s @30FPS

// --- Optical-Flow-Richtungs-Gate (nur Tap-Zonen, nie down) ---
// Ein Tap feuert nur, wenn die Bewegung in der Zone auch in Gestenrichtung
// zeigt (left: nach aussen links, right: nach aussen rechts, up: aufwaerts).
// Durchlaufende Personen / Vorbeugen loesen so nicht mehr aus.
pub const FLOW_ENABLED: bool = true;
pub const FLOW_WIDTH: i32 = 160; // Flow auf stark verkleinertem Graubild
pub const FLOW_HEIGHT: i32 = 120;
pub const FLOW_MIN_MAG: f64 = 0.5; // Mindestbetrag in px/Frame auf 160x120
pub const FLOW_MIN_PIXELS: i32 = 5; // min. Vordergrund-Pixel je Zone fuer ein Urteil
pub const FLOW_MEMORY_FRAMES: u32 = 3; // Richtung gilt fuer die naechsten N Frames

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
        enter_ratio: ZONE_ENTER_RATIOS,
        exit_ratio: ZONE_EXIT_RATIOS,
        cooldown_s: COOLDOWN_S,
        confirm_frames: CONFIRM_FRAMES,
    }
}
