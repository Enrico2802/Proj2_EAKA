"""Beweis-Screen / Overlay (KONZEPT Abs. 8) - ein OpenCV-Fenster auf Monitor 2.

Zeigt das Kamerabild ODER die Maske (Taste 'm'), das Raster, aktive Zellen,
die Zonenrahmen, die gerade getriggerte Zone hervorgehoben und gross die
aktuell gedrueckte Taste. Liefert pro Frame die gedrueckte Steuertaste zurueck.
"""

import time

import cv2
import numpy as np

import config

WIN = "Webcam-Steuerung (Beweis-Screen)"

# Anzeige-Text je Taste
_KEY_LABEL = {
    config.KEYS["left"]:  "<- A",
    config.KEYS["right"]: "D ->",
    config.KEYS["up"]:    "[ SPRUNG ]",
    config.KEYS["down"]:  "v S",
}


class Overlay:
    def __init__(self):
        self.show_mask = False
        self._last_t = None
        self._fps = 0.0
        cv2.namedWindow(WIN, cv2.WINDOW_NORMAL)
        cv2.moveWindow(WIN, config.MONITOR2_X_OFFSET, 0)

    def _base_image(self, s):
        if self.show_mask and s.grid is not None:
            # aktive Zellen als graue Maske hochskalieren
            mask = (s.grid.astype(np.uint8) * 255)
            img = cv2.resize(mask, (config.FRAME_WIDTH, config.FRAME_HEIGHT),
                             interpolation=cv2.INTER_NEAREST)
            return cv2.cvtColor(img, cv2.COLOR_GRAY2BGR)
        if s.frame is not None:
            return s.frame.copy()
        return np.zeros((config.FRAME_HEIGHT, config.FRAME_WIDTH, 3), np.uint8)

    def show(self, s, detector, current_key, triggered_zone) -> str:
        img = self._base_image(s)
        h, w = img.shape[:2]

        # --- aktive Zellen halbtransparent einfaerben ---
        if s.grid is not None:
            overlay_layer = img.copy()
            rows, cols = s.grid.shape
            cw, ch = w / cols, h / rows
            ys, xs = np.where(s.grid)
            for y, x in zip(ys, xs):
                p0 = (int(x * cw), int(y * ch))
                p1 = (int((x + 1) * cw), int((y + 1) * ch))
                cv2.rectangle(overlay_layer, p0, p1, (0, 200, 255), -1)
            cv2.addWeighted(overlay_layer, 0.35, img, 0.65, 0, img)

        # --- Raster-Linien ---
        for c in range(1, config.GRID_COLS):
            x = int(c * w / config.GRID_COLS)
            cv2.line(img, (x, 0), (x, h), (60, 60, 60), 1)
        for r in range(1, config.GRID_ROWS):
            y = int(r * h / config.GRID_ROWS)
            cv2.line(img, (0, y), (w, y), (60, 60, 60), 1)

        # --- Zonenrahmen + getriggerte Zone hervorheben ---
        for name, (x0, y0, x1, y1) in config.ZONES.items():
            p0 = (int(x0 * w), int(y0 * h))
            p1 = (int(x1 * w), int(y1 * h))
            triggered = (name == triggered_zone)
            color = (0, 0, 255) if triggered else (0, 255, 0)
            thick = 4 if triggered else 1
            cv2.rectangle(img, p0, p1, color, thick)
            cv2.putText(img, name, (p0[0] + 4, p0[1] + 18),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.5, color, 1, cv2.LINE_AA)

        # --- grosse Tastenanzeige ---
        if current_key:
            label = _KEY_LABEL.get(current_key, current_key.upper())
            cv2.putText(img, label, (int(w * 0.30), 50),
                        cv2.FONT_HERSHEY_SIMPLEX, 1.4, (0, 0, 255), 3, cv2.LINE_AA)

        # --- FPS ---
        now = time.monotonic()
        if self._last_t is not None:
            dt = now - self._last_t
            if dt > 0:
                self._fps = 0.9 * self._fps + 0.1 * (1.0 / dt)
        self._last_t = now

        # --- Statuszeile ---
        status = (f"FPS {self._fps:4.1f} | Grid {config.GRID_COLS}x{config.GRID_ROWS} "
                  f"| enter {detector.enter_ratio:.2f} exit {detector.exit_ratio:.2f} "
                  f"| {'KALIBRIERT' if detector.calibrated else 'kalibriere...'} "
                  f"| m=Maske c=Kalib q=Ende")
        cv2.putText(img, status, (6, h - 8),
                    cv2.FONT_HERSHEY_SIMPLEX, 0.45, (255, 255, 255), 1, cv2.LINE_AA)

        cv2.imshow(WIN, img)
        key = cv2.waitKey(1) & 0xFF
        if key in (ord("q"),):
            return "q"
        if key == 27:
            return "esc"
        if key == ord("m"):
            self.show_mask = not self.show_mask
        if key == ord("c"):
            return "c"
        return ""

    def close(self):
        cv2.destroyAllWindows()
