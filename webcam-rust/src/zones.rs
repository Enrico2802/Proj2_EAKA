//! Daten-Vertrag zwischen Quelle und Detector (Port von zones.py).
//!
//! Eine Quelle liefert pro Frame die Bewegungs-ANTEILE je Zone (0.0..1.0).
//! Der Detector liest ausschliesslich diese Werte + den Zeitstempel und ist
//! damit ohne Kamera testbar.

/// Anteil aktiver Zellen je Zone, jeweils 0.0..1.0.
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

/// Ein Frame aus Sicht des Detectors: Zonen-Anteile + Zeitstempel (Sekunden).
/// Bild/Maske fuer das Overlay werden im Webcam-Layer separat gefuehrt, damit
/// dieser Typ frei von OpenCV bleibt.
#[derive(Debug, Clone, Copy)]
pub struct ZoneActivity {
    pub zones: Zones,
    pub t: f64,
}

impl ZoneActivity {
    pub fn new(zones: Zones, t: f64) -> Self {
        Self { zones, t }
    }
}
