"""Echte Quelle: Webcam -> MOG2-Vordergrundmaske -> Raster -> Zonen-Anteile.

Verarbeitungskette pro Frame (KONZEPT Abs. 5):
    Frame holen -> optional spiegeln -> MOG2-Maske -> Schatten weg -> Morphologie
    -> auf GRID_COLS x GRID_ROWS skalieren -> aktive Zellen -> Anteil je Zone.

Liefert dieselben ZoneActivity-Objekte wie Mock/Manual, zusaetzlich frame + grid
fuer das Overlay.
"""

import time

import cv2
import numpy as np

import config
from zones import ZoneActivity, ZONE_NAMES


class WebcamGridSource:
    def __init__(self, camera_index: int = None, mirror: bool = None):
        self.camera_index = config.CAMERA_INDEX if camera_index is None else camera_index
        self.mirror = config.MIRROR if mirror is None else mirror

        self.cap = cv2.VideoCapture(self.camera_index, cv2.CAP_DSHOW)
        if not self.cap.isOpened():
            # Fallback ohne DSHOW-Backend
            self.cap = cv2.VideoCapture(self.camera_index)
        if not self.cap.isOpened():
            raise RuntimeError(f"Kamera {self.camera_index} konnte nicht geoeffnet werden.")
        self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, config.FRAME_WIDTH)
        self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, config.FRAME_HEIGHT)

        self._kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (3, 3))
        self._make_subtractor()
        self._t0 = None

    def _make_subtractor(self):
        self.backsub = cv2.createBackgroundSubtractorMOG2(
            history=config.MOG2_HISTORY,
            varThreshold=config.MOG2_VAR_THRESHOLD,
            detectShadows=config.MOG2_DETECT_SHADOWS,
        )

    def recalibrate(self):
        """Hintergrundmodell neu lernen (Licht/Standort hat sich geaendert)."""
        self._make_subtractor()

    def _zone_ratios(self, active: np.ndarray) -> dict:
        rows, cols = active.shape
        zones = {}
        for name in ZONE_NAMES:
            x0, y0, x1, y1 = config.ZONES[name]
            c0, c1 = int(x0 * cols), int(x1 * cols)
            r0, r1 = int(y0 * rows), int(y1 * rows)
            sub = active[r0:r1, c0:c1]
            zones[name] = float(sub.mean()) if sub.size else 0.0
        return zones

    def frames(self):
        while True:
            ret, frame = self.cap.read()
            if not ret:
                continue
            if self._t0 is None:
                self._t0 = time.monotonic()
            t = time.monotonic() - self._t0

            if self.mirror:
                frame = cv2.flip(frame, 1)

            # (4) MOG2-Vordergrundmaske
            fgmask = self.backsub.apply(frame)
            # (5) Schatten (Wert 127) entfernen -> nur harter Vordergrund
            _, fgmask = cv2.threshold(fgmask, 200, 255, cv2.THRESH_BINARY)
            # (6) Morphologie: Rauschen weg, Loecher zu
            fgmask = cv2.morphologyEx(fgmask, cv2.MORPH_OPEN, self._kernel)
            fgmask = cv2.morphologyEx(fgmask, cv2.MORPH_CLOSE, self._kernel)
            # (7) Rasterung
            small = cv2.resize(fgmask, (config.GRID_COLS, config.GRID_ROWS),
                               interpolation=cv2.INTER_AREA)
            # (8) aktive Zellen
            active = small >= config.CELL_ACTIVE_THRESH
            # (9) Zonen-Anteile
            zones = self._zone_ratios(active)

            yield ZoneActivity(zones=zones, t=t, frame=frame, grid=active)

    def close(self):
        if self.cap is not None:
            self.cap.release()
            self.cap = None
