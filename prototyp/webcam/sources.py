"""Data sources with a uniform interface.

Each source is iterable via frames() and yields ZoneActivity objects (data
contract from zones.py). MockSource and ManualSource need no camera and no
extra packages; WebcamGridSource (in webcam_source.py) delivers the same
objects from real camera images.
"""

import math
import time

from zones import ZoneActivity, full_zones

FPS = 30
DT = 1.0 / FPS


class MockSource:
    """Plays back a fixed script of zone activities (concept document, section 4.2a)."""

    def __init__(self, realtime: bool = True):
        self.realtime = realtime   # False = as fast as possible (tests)
        self._t = 0.0

    def _frame(self, zones: dict) -> ZoneActivity:
        s = ZoneActivity(zones=full_zones(zones), t=self._t)
        self._t += DT
        if self.realtime:
            time.sleep(DT)
        return s

    def _hold(self, zones: dict, seconds: float):
        for _ in range(int(seconds * FPS)):
            yield self._frame(zones)

    def frames(self):
        print(">> Mock: Ruhe (Kalibrierung)...")
        yield from self._hold({}, 1.5)
        print(">> Mock: Arme HOCH (oben Mitte) -> Sprung")
        yield from self._hold({"up": 0.6}, 0.3)
        yield from self._hold({}, 1.0)
        print(">> Mock: Arm nach LINKS -> A")
        yield from self._hold({"left": 0.7}, 0.3)
        yield from self._hold({}, 1.0)
        print(">> Mock: Arm nach RECHTS -> D")
        yield from self._hold({"right": 0.7}, 0.3)
        yield from self._hold({}, 1.0)
        print(">> Mock: DUCKEN (unten, 1.2s) -> HOLD S")
        yield from self._hold({"down": 0.9}, 1.2)
        print(">> Mock: aufstehen -> HOLD S aus")
        yield from self._hold({}, 1.0)
        print(">> Mock: LINKS und rechts gleichzeitig -> nur A (staerker)")
        yield from self._hold({"left": 0.7, "right": 0.3}, 0.3)
        yield from self._hold({}, 0.5)
        print(">> Mock: beendet.")

    def recalibrate(self):
        pass

    def close(self):
        pass


class ManualSource:
    """Simulate a person via keyboard: a/d/w create zone activity, s toggles
    the hold zone 'down'. q quits. Keys are read in the console window
    (msvcrt) - therefore use with dry run (concept document, section 4.2b)."""

    def frames(self):
        import msvcrt
        print("Steuerung: [w]=oben/Sprung  [a]=links  [d]=rechts  "
              "[s]=ducken an/aus  [c]=neu kalibrieren  [q]=Ende")
        t = 0.0
        down = False
        while True:
            zones = {"left": 0.0, "right": 0.0, "up": 0.0, "down": 0.0}
            while msvcrt.kbhit():
                key = msvcrt.getwch().lower()
                if key == "q":
                    return
                elif key == "w":
                    zones["up"] = 0.9
                elif key == "a":
                    zones["left"] = 0.9
                elif key == "d":
                    zones["right"] = 0.9
                elif key == "s":
                    down = not down
                elif key == "c":
                    self._recalib = True
            if down:
                zones["down"] = 0.9
            yield ZoneActivity(zones=zones, t=t)
            t += DT
            time.sleep(DT)

    def recalibrate(self):
        pass

    def close(self):
        pass
