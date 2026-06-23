"""Zentrale Konfiguration der Webcam-Steuerung.

Reine Konstanten - keine Logik. Alle anderen Module importieren von hier.
Die Schwellwerte sind ANTEILIG (Prozentanteil aktiver Zellen je Zone) und
damit unabhaengig von der Rastergroesse (siehe KONZEPT Abs. 4.1).
"""

# --- Kamera / Bild ---
CAMERA_INDEX        = 0      # OpenCV-Index (nicht die USB-VID/PID!). 0 = Standardkamera.
# FullHD laeuft mit ~30 FPS (Pipeline inkl. MOG2 gemessen). Voraussetzung ist das
# MJPG-Format, das webcam_source.py erzwingt - sonst faellt die Cam auf YUY2 zurueck
# (~5 FPS). Alternativen: 1280x720 (sparsam), 2560x1440 (~27 FPS).
FRAME_WIDTH         = 1920
FRAME_HEIGHT        = 1080
MIRROR              = True   # Selfie-Ansicht: links im Bild = links der Person

# --- Raster ---
GRID_COLS           = 32
GRID_ROWS           = 24
CELL_ACTIVE_THRESH  = 40     # Maskenwert 0..255, ab wann eine Zelle als "aktiv" gilt

# --- Detector-Schwellen (Anteile 0..1) ---
ZONE_ENTER_RATIO    = 0.15   # Anteil aktiver Zellen -> Zone "an"
ZONE_EXIT_RATIO     = 0.08   # Hysterese -> Zone "aus"
COOLDOWN_S          = 0.5    # Sperrzeit je Tap-Geste gegen Doppelausloesung
CALIBRATION_S       = 1.0    # Dauer der Start-Kalibrierung
FPS_ANNAHME         = 30     # nur zur Umrechnung CALIBRATION_S -> calib_frames

# --- MOG2 Hintergrund-Subtraktion ---
MOG2_HISTORY        = 300
MOG2_VAR_THRESHOLD  = 25
MOG2_DETECT_SHADOWS = True

# --- KeySender ---
TAP_HOLD_MS         = 40     # Haltedauer eines Taps in Millisekunden

# --- Overlay ---
MONITOR2_X_OFFSET   = 1920   # x-Position des Overlay-Fensters (2. Monitor)

# --- Geste -> Taste ---
KEYS = {"left": "a", "right": "d", "up": "space", "down": "s"}

# Zonen als relative Rechtecke (0..1), Format (x0, y0, x1, y1).
# NICHT ueberlappen lassen! Mittleres Feld bleibt neutrale Ruhezone.
ZONES = {
    "up":    (0.34, 0.00, 0.66, 0.38),   # oben Mitte   -> Sprung (Leertaste)
    "left":  (0.00, 0.00, 0.34, 0.70),   # links        -> A
    "right": (0.66, 0.00, 1.00, 0.70),   # rechts       -> D
    "down":  (0.00, 0.70, 1.00, 1.00),   # unteres Band -> S (ducken, HOLD)
}

# abgeleitet
CALIB_FRAMES = round(CALIBRATION_S * FPS_ANNAHME)
