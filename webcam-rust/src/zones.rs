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
///
/// `dir_ok` ist das Richtungs-Gate der Quelle fuer die Tap-Zonen
/// [left, right, up]: true = Bewegung zeigt in Gestenrichtung (oder die
/// Quelle liefert keine Richtungsinfo, dann bleibt das Gate offen). Die
/// down-Zone hat bewusst kein Gate - Halten bedeutet Stillstand, Flow ~0.
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

    /// Variante fuer Quellen mit Richtungsinfo (Optical Flow).
    pub fn with_dir_ok(mut self, dir_ok: [bool; 3]) -> Self {
        self.dir_ok = dir_ok;
        self
    }
}
