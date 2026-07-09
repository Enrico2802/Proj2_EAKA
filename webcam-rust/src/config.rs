//! Central configuration (port of config.py). Constants only.

use crate::detector::Config as DetectorConfig;
use crate::keysender::Key;

// --- Camera / image ---
pub const CAMERA_INDEX: i32 = 0;
pub const FRAME_WIDTH: i32 = 1920;
pub const FRAME_HEIGHT: i32 = 1080;
pub const MIRROR: bool = true;

// --- Grid ---
pub const GRID_COLS: i32 = 32;
pub const GRID_ROWS: i32 = 24;
pub const CELL_ACTIVE_THRESH: f64 = 40.0; // mask value 0..255

// --- Detector thresholds (ratios 0..1) ---
// Per zone [left, right, up, down]: the gestures fill their zones very
// differently (thin arm in the small up zone vs. the whole body in the down
// band), so a single shared threshold never fits all of them.
// down exit deliberately low: during a long hold the MOG2 mask fades
// gradually, a high exit would end the hold too early.
pub const ZONE_ENTER_RATIOS: [f64; 4] = [0.15, 0.15, 0.10, 0.22];
pub const ZONE_EXIT_RATIOS: [f64; 4] = [0.08, 0.08, 0.05, 0.08];
pub const COOLDOWN_S: f64 = 0.5;
pub const CALIB_FRAMES: usize = 30; // ~1s @30FPS
// An edge counts only after N consecutive frames above/below the threshold
// (debounce against single-frame outliers; 1 = immediate).
pub const CONFIRM_FRAMES: u32 = 2;

// --- MOG2 ---
pub const MOG2_HISTORY: i32 = 300;
pub const MOG2_VAR_THRESHOLD: f64 = 25.0;
pub const MOG2_DETECT_SHADOWS: bool = true;
// After the warmup the background model is frozen (learning rate 0.0),
// otherwise a person standing still / crouching gets absorbed into the model
// within ~10-25s and a held S aborts with a spurious HoldDownEnd.
// Lighting changes are handled by auto-recalibration (below) and the 'c' key.
pub const MOG2_WARMUP_FRAMES: u32 = 60; // ~2s @30FPS, covers CALIB_FRAMES
pub const MOG2_FROZEN_LEARNING_RATE: f64 = 0.0;

// --- Auto-recalibration on scene change (lights on/off, camera moved) ---
// If a large share of ALL grid cells stays active for a longer period, that
// is not a gesture but a global lighting change.
pub const SCENE_CHANGE_RATIO: f64 = 0.6;
pub const SCENE_CHANGE_FRAMES: u32 = 45; // ~1.5s @30FPS

// --- Optical-flow direction gate (tap zones only, never down) ---
// A tap only fires if the motion inside the zone also points in gesture
// direction (left: outward left, right: outward right, up: upward).
// People walking through / leaning forward no longer trigger.
pub const FLOW_ENABLED: bool = true;
pub const FLOW_WIDTH: i32 = 160; // flow runs on a strongly downscaled gray image
pub const FLOW_HEIGHT: i32 = 120;
pub const FLOW_MIN_MAG: f64 = 0.5; // minimum magnitude in px/frame at 160x120
pub const FLOW_MIN_PIXELS: i32 = 5; // min. foreground pixels per zone for a verdict
pub const FLOW_MEMORY_FRAMES: u32 = 3; // direction stays valid for the next N frames

// --- KeySender ---
pub const TAP_HOLD_MS: u64 = 40;

// --- Overlay ---
pub const MONITOR2_X_OFFSET: i32 = 1920; // x position of the overlay window (2nd monitor)

/// Zone rectangle in relative image coordinates (0..1).
#[derive(Debug, Clone, Copy)]
pub struct ZoneRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// Zones in detector index order: [left, right, up, down].
/// NON-overlapping; the center area stays a neutral idle zone.
pub const ZONE_RECTS: [ZoneRect; 4] = [
    ZoneRect { x0: 0.00, y0: 0.00, x1: 0.34, y1: 0.70 }, // left  -> A
    ZoneRect { x0: 0.66, y0: 0.00, x1: 1.00, y1: 0.70 }, // right -> D
    ZoneRect { x0: 0.34, y0: 0.00, x1: 0.66, y1: 0.38 }, // up    -> W
    ZoneRect { x0: 0.00, y0: 0.70, x1: 1.00, y1: 1.00 }, // down  -> S (Hold)
];

/// Gesture -> key.
pub const KEY_LEFT: Key = Key::A;
pub const KEY_RIGHT: Key = Key::D;
pub const KEY_UP: Key = Key::W;
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
