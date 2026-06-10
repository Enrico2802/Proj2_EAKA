"""Simulierte Person vor der Kinect.

Spielt ein festes "Drehbuch" ab: stehen (Kalibrierung), springen, Schritt
nach links, zurück, ducken, Schritt nach rechts, zurück, springen.
So lässt sich die komplette Pipeline ohne Hardware testen und vorführen.

Später wird diese Klasse durch eine Freenect2Source ersetzt, die dieselben
BodyState-Frames aus echten Kinect-Depth-Daten berechnet.
"""

import math
import random
import time

from body_state import BodyState

FPS = 30
DT = 1.0 / FPS

STAND_HEIGHT = 1.75   # m - Körpergröße der simulierten Person
JUMP_PEAK = 0.30      # m - wie hoch der Schwerpunkt beim Sprung steigt
CROUCH_HEIGHT = 1.30  # m - Körperhöhe in der Hocke
SIDE_X = 0.55         # normierter x-Versatz einer Seitenspur


class MockSource:
    def __init__(self, realtime: bool = True):
        self.realtime = realtime  # False = so schnell wie möglich (für Tests)
        self._t = 0.0
        self._x = 0.0

    def _frame(self, x: float, height: float) -> BodyState:
        s = BodyState(
            x=x + random.uniform(-0.01, 0.01),          # Sensor-Rauschen
            height=height + random.uniform(-0.01, 0.01),
            t=self._t,
        )
        self._t += DT
        if self.realtime:
            time.sleep(DT)
        return s

    def _hold(self, seconds: float, height: float = STAND_HEIGHT):
        for _ in range(int(seconds * FPS)):
            yield self._frame(self._x, height)

    def _jump(self, duration: float = 0.45):
        n = int(duration * FPS)
        for i in range(n):
            p = i / (n - 1)
            h = STAND_HEIGHT + JUMP_PEAK * math.sin(math.pi * p)  # Parabel rauf und runter
            yield self._frame(self._x, h)

    def _step_to(self, target_x: float, duration: float = 0.4):
        n = int(duration * FPS)
        start = self._x
        for i in range(n):
            self._x = start + (target_x - start) * (i + 1) / n
            yield self._frame(self._x, STAND_HEIGHT)

    def frames(self):
        print(">> Simulation: Person steht still (Kalibrierung)...")
        yield from self._hold(1.5)
        print(">> Simulation: SPRUNG")
        yield from self._jump()
        yield from self._hold(1.0)
        print(">> Simulation: Schritt nach LINKS")
        yield from self._step_to(-SIDE_X)
        yield from self._hold(0.8)
        print(">> Simulation: zurück zur Mitte")
        yield from self._step_to(0.0)
        yield from self._hold(0.8)
        print(">> Simulation: DUCKEN (1.2s)")
        yield from self._hold(1.2, height=CROUCH_HEIGHT)
        print(">> Simulation: wieder aufstehen")
        yield from self._hold(1.0)
        print(">> Simulation: Schritt nach RECHTS")
        yield from self._step_to(+SIDE_X)
        yield from self._hold(0.8)
        print(">> Simulation: zurück zur Mitte")
        yield from self._step_to(0.0)
        yield from self._hold(0.5)
        print(">> Simulation: noch ein SPRUNG")
        yield from self._jump()
        yield from self._hold(1.0)
        print(">> Simulation beendet.")
