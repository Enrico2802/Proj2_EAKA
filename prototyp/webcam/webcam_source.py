"""Real source: webcam -> MOG2 foreground mask -> grid -> zone ratios.

Processing chain per frame (concept document, section 5):
    grab frame -> optional mirror -> MOG2 mask -> remove shadows -> morphology
    -> resize to GRID_COLS x GRID_ROWS -> active cells -> ratio per zone.

Delivers the same ZoneActivity objects as mock/manual, plus frame + grid
for the overlay.
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
            # fallback without the DSHOW backend
            self.cap = cv2.VideoCapture(self.camera_index)
        if not self.cap.isOpened():
            raise RuntimeError(f"Kamera {self.camera_index} konnte nicht geoeffnet werden.")
        # IMPORTANT for high resolutions: force MJPG, otherwise the cam delivers
        # uncompressed YUY2 and the USB bandwidth pushes the FPS down to ~5.
        # FOURCC must be set BEFORE AND AFTER the resolution (driver quirk).
        mjpg = cv2.VideoWriter_fourcc(*"MJPG")
        self.cap.set(cv2.CAP_PROP_FOURCC, mjpg)
        self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, config.FRAME_WIDTH)
        self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, config.FRAME_HEIGHT)
        self.cap.set(cv2.CAP_PROP_FOURCC, mjpg)
        actual_w = int(self.cap.get(cv2.CAP_PROP_FRAME_WIDTH))
        actual_h = int(self.cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
        print(f"Kamera {self.camera_index}: {actual_w}x{actual_h}")

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
        """Relearn the background model (lighting/position has changed)."""
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

            # Build a hard foreground mask from MOG2.
            fgmask = self.backsub.apply(frame)
            _, fgmask = cv2.threshold(fgmask, 200, 255, cv2.THRESH_BINARY)
            # Remove noise and close small holes.
            fgmask = cv2.morphologyEx(fgmask, cv2.MORPH_OPEN, self._kernel)
            fgmask = cv2.morphologyEx(fgmask, cv2.MORPH_CLOSE, self._kernel)
            small = cv2.resize(fgmask, (config.GRID_COLS, config.GRID_ROWS),
                               interpolation=cv2.INTER_AREA)
            active = small >= config.CELL_ACTIVE_THRESH
            zones = self._zone_ratios(active)

            yield ZoneActivity(zones=zones, t=t, frame=frame, grid=active)

    def close(self):
        if self.cap is not None:
            self.cap.release()
            self.cap = None
