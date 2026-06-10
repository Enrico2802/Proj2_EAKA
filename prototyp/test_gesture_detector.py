"""Unit-Tests für die Gestenerkennung (python -m unittest)."""

import unittest

from body_state import BodyState
from gesture_detector import GestureDetector

FPS = 30
DT = 1.0 / FPS
STAND = 1.75


class GestureDetectorTest(unittest.TestCase):
    def setUp(self):
        self.det = GestureDetector(calib_frames=10)
        self.t = 0.0

    def feed(self, x: float, height: float, frames: int = 1) -> list[str]:
        """Schickt Frames durch den Detektor und sammelt alle Events."""
        events = []
        for _ in range(frames):
            events += self.det.update(BodyState(x=x, height=height, t=self.t))
            self.t += DT
        return events

    def calibrate(self):
        self.feed(0.0, STAND, frames=10)
        self.assertTrue(self.det.calibrated)

    def test_keine_events_waehrend_kalibrierung(self):
        events = self.feed(0.0, STAND, frames=9)
        self.assertEqual(events, [])
        self.assertFalse(self.det.calibrated)

    def test_sprung_feuert_genau_einmal(self):
        self.calibrate()
        events = self.feed(0.0, STAND + 0.25, frames=8)   # in der Luft
        events += self.feed(0.0, STAND, frames=5)          # gelandet
        self.assertEqual(events.count("jump"), 1)

    def test_sprung_cooldown_blockt_doppelausloesung(self):
        self.calibrate()
        events = self.feed(0.0, STAND + 0.25, frames=3)
        events += self.feed(0.0, STAND, frames=2)          # landet sofort wieder
        events += self.feed(0.0, STAND + 0.25, frames=3)   # zweiter "Sprung" innerhalb des Cooldowns
        self.assertEqual(events.count("jump"), 1)

    def test_zwei_spruenge_nach_cooldown(self):
        self.calibrate()
        events = self.feed(0.0, STAND + 0.25, frames=5)
        events += self.feed(0.0, STAND, frames=20)         # > 0.5s Cooldown abwarten
        events += self.feed(0.0, STAND + 0.25, frames=5)
        self.assertEqual(events.count("jump"), 2)

    def test_ducken_start_und_ende(self):
        self.calibrate()
        events = self.feed(0.0, STAND - 0.40, frames=10)
        self.assertEqual(events.count("crouch_start"), 1)
        self.assertEqual(events.count("crouch_end"), 0)
        events = self.feed(0.0, STAND, frames=5)
        self.assertEqual(events.count("crouch_end"), 1)

    def test_spurwechsel_links_und_zurueck(self):
        self.calibrate()
        events = self.feed(-0.50, STAND, frames=5)
        self.assertEqual(events.count("lane_left"), 1)
        events = self.feed(0.0, STAND, frames=5)
        self.assertEqual(events.count("lane_right"), 1)

    def test_hysterese_kein_flackern_an_der_spurgrenze(self):
        self.calibrate()
        # Knapp über der Eintritts-Schwelle, dann knapp darunter (aber über der Austritts-Schwelle):
        events = self.feed(-0.30, STAND, frames=3)
        events += self.feed(-0.20, STAND, frames=3)
        events += self.feed(-0.30, STAND, frames=3)
        self.assertEqual(events.count("lane_left"), 1)
        self.assertEqual(events.count("lane_right"), 0)

    def test_kleine_bewegungen_loesen_nichts_aus(self):
        self.calibrate()
        events = self.feed(0.05, STAND + 0.03, frames=30)
        self.assertEqual(events, [])


if __name__ == "__main__":
    unittest.main()
