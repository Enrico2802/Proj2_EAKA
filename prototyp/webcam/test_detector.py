"""Unit tests of the gesture detection (python -m unittest), without camera.

Deterministic and CI-capable - exactly these cases form the reference for
a later Rust port (concept document, section 10).
"""

import unittest

from zones import ZoneActivity, full_zones
from detector import GestureDetector

FPS = 30
DT = 1.0 / FPS


class DetectorTest(unittest.TestCase):
    def setUp(self):
        # short calibration for fast tests; thresholds = defaults
        self.det = GestureDetector(calib_frames=10)
        self.t = 0.0

    def feed(self, zones, frames=1):
        ev = []
        for _ in range(frames):
            ev += self.det.update(ZoneActivity(zones=full_zones(zones), t=self.t))
            self.t += DT
        return ev

    def calibrate(self):
        self.feed({}, frames=10)
        self.assertTrue(self.det.calibrated)

    def test_keine_events_waehrend_kalibrierung(self):
        ev = self.feed({}, frames=9)
        self.assertEqual(ev, [])
        self.assertFalse(self.det.calibrated)

    def test_right_feuert_genau_einmal(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=8)
        self.assertEqual(ev.count("tap_right"), 1)

    def test_cooldown_blockt_doppelausloesung(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=3)
        ev += self.feed({}, frames=2)               # briefly out (< exit)
        ev += self.feed({"right": 0.7}, frames=3)   # in again, but < 0.5 s
        self.assertEqual(ev.count("tap_right"), 1)

    def test_zwei_taps_nach_cooldown(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=3)
        ev += self.feed({}, frames=18)              # wait > 0.5 s
        ev += self.feed({"right": 0.7}, frames=3)
        self.assertEqual(ev.count("tap_right"), 2)

    def test_hold_down_start_und_end(self):
        self.calibrate()
        ev = self.feed({"down": 0.9}, frames=10)
        self.assertEqual(ev.count("hold_down_start"), 1)
        self.assertEqual(ev.count("hold_down_end"), 0)
        ev = self.feed({}, frames=5)
        self.assertEqual(ev.count("hold_down_end"), 1)

    def test_hysterese_kein_flackern(self):
        self.calibrate()
        ev = self.feed({"left": 0.16}, frames=3)    # enter (> 0.15)
        ev += self.feed({"left": 0.10}, frames=3)   # inside the band (0.08..0.15): stays active
        ev += self.feed({"left": 0.16}, frames=3)   # still active -> no new tap
        self.assertEqual(ev.count("tap_left"), 1)

    def test_konflikt_staerkere_zone_gewinnt(self):
        self.calibrate()
        ev = self.feed({"left": 0.7, "right": 0.3}, frames=3)
        self.assertEqual(ev.count("tap_left"), 1)
        self.assertEqual(ev.count("tap_right"), 0)

    def test_kleine_bewegungen_loesen_nichts_aus(self):
        self.calibrate()
        ev = self.feed({"left": 0.05, "right": 0.04, "up": 0.03, "down": 0.02}, frames=30)
        self.assertEqual(ev, [])

    def test_up_feuert_tap(self):
        self.calibrate()
        ev = self.feed({"up": 0.6}, frames=6)
        self.assertEqual(ev.count("tap_up"), 1)


if __name__ == "__main__":
    unittest.main()
