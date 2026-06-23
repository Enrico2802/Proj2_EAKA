"""Unit-Tests der Gestenerkennung (python -m unittest), ohne Kamera.

Deterministisch und CI-faehig - genau diese Faelle bilden die Referenz fuer
einen spaeteren Rust-Port (KONZEPT Abs. 10, T1..T9).
"""

import unittest

from zones import ZoneActivity, full_zones
from detector import GestureDetector

FPS = 30
DT = 1.0 / FPS


class DetectorTest(unittest.TestCase):
    def setUp(self):
        # kleine Kalibrierung fuer schnelle Tests; Schwellen = Defaults
        self.det = GestureDetector(calib_frames=10)
        self.t = 0.0

    def feed(self, zones, frames=1):
        ev = []
        for _ in range(frames):
            ev += self.det.update(ZoneActivity(zones=full_zones(zones), t=self.t))
            self.t += DT
        return ev

    def calibrate(self):
        self.feed({}, frames=10)   # alle Zonen 0.0
        self.assertTrue(self.det.calibrated)

    # T1
    def test_keine_events_waehrend_kalibrierung(self):
        ev = self.feed({}, frames=9)
        self.assertEqual(ev, [])
        self.assertFalse(self.det.calibrated)

    # T2
    def test_right_feuert_genau_einmal(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=8)
        self.assertEqual(ev.count("tap_right"), 1)

    # T3
    def test_cooldown_blockt_doppelausloesung(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=3)
        ev += self.feed({}, frames=2)               # kurz raus (< exit)
        ev += self.feed({"right": 0.7}, frames=3)   # erneut rein, aber < 0.5s
        self.assertEqual(ev.count("tap_right"), 1)

    # T4
    def test_zwei_taps_nach_cooldown(self):
        self.calibrate()
        ev = self.feed({"right": 0.7}, frames=3)
        ev += self.feed({}, frames=18)              # > 0.5s warten
        ev += self.feed({"right": 0.7}, frames=3)
        self.assertEqual(ev.count("tap_right"), 2)

    # T5
    def test_hold_down_start_und_end(self):
        self.calibrate()
        ev = self.feed({"down": 0.9}, frames=10)
        self.assertEqual(ev.count("hold_down_start"), 1)
        self.assertEqual(ev.count("hold_down_end"), 0)
        ev = self.feed({}, frames=5)
        self.assertEqual(ev.count("hold_down_end"), 1)

    # T6
    def test_hysterese_kein_flackern(self):
        self.calibrate()
        ev = self.feed({"left": 0.16}, frames=3)    # rein (> 0.15)
        ev += self.feed({"left": 0.10}, frames=3)   # im Band (0.08..0.15): bleibt aktiv
        ev += self.feed({"left": 0.16}, frames=3)   # immer noch aktiv -> kein neuer Tap
        self.assertEqual(ev.count("tap_left"), 1)

    # T7
    def test_konflikt_staerkere_zone_gewinnt(self):
        self.calibrate()
        ev = self.feed({"left": 0.7, "right": 0.3}, frames=3)
        self.assertEqual(ev.count("tap_left"), 1)
        self.assertEqual(ev.count("tap_right"), 0)

    # T8
    def test_kleine_bewegungen_loesen_nichts_aus(self):
        self.calibrate()
        ev = self.feed({"left": 0.05, "right": 0.04, "up": 0.03, "down": 0.02}, frames=30)
        self.assertEqual(ev, [])

    # T9
    def test_up_feuert_tap(self):
        self.calibrate()
        ev = self.feed({"up": 0.6}, frames=6)
        self.assertEqual(ev.count("tap_up"), 1)


if __name__ == "__main__":
    unittest.main()
