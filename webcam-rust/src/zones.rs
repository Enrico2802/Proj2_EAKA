//! Data contract between source and detector (port of zones.py).
//!
//! A source delivers the motion RATIOS per zone (0.0..1.0) for each frame.
//! The detector reads only these values + the timestamp and is therefore
//! testable without a camera.

/// Ratio of active cells per zone, each 0.0..1.0.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Zones {
    pub left: f64,
    pub right: f64,
    pub up: f64,
    pub down: f64,
}

impl Zones {
    pub fn new(left: f64, right: f64, up: f64, down: f64) -> Self {
        Self { left, right, up, down }
    }
}

/// One frame from the detector's point of view: zone ratios + timestamp
/// (seconds). Image/mask for the overlay are kept separately in the webcam
/// layer so this type stays free of OpenCV.
///
/// `dir_ok` is the source's direction gate for the tap zones
/// [left, right, up]: true = motion points in gesture direction (or the
/// source provides no direction info, then the gate stays open). The down
/// zone deliberately has no gate - holding means standing still, flow ~0.
#[derive(Debug, Clone, Copy)]
pub struct ZoneActivity {
    pub zones: Zones,
    pub t: f64,
    pub dir_ok: [bool; 3],
}

impl ZoneActivity {
    pub fn new(zones: Zones, t: f64) -> Self {
        Self { zones, t, dir_ok: [true; 3] }
    }

    /// Variant for sources with direction info (optical flow).
    pub fn with_dir_ok(mut self, dir_ok: [bool; 3]) -> Self {
        self.dir_ok = dir_ok;
        self
    }
}
