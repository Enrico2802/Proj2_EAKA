"""Simulated person in front of the Kinect.

Plays back a fixed "script": stand (calibration), jump, step left, back,
crouch, step right, back, jump. This allows testing and demonstrating the
complete pipeline without hardware.

Later this class gets replaced by a Freenect2Source that computes the same
BodyState frames from real Kinect depth data.
"""

import math
import random
import time

from body_state import BodyState

FPS = 30
DT = 1.0 / FPS

STAND_HEIGHT = 1.75   # m - body height of the simulated person
JUMP_PEAK = 0.30      # m - how far the centroid rises during a jump
CROUCH_HEIGHT = 1.30  # m - body height while crouching
SIDE_X = 0.55         # normalized x offset of a side lane


class MockSource:
    def __init__(self, realtime: bool = True):
        self.realtime = realtime  # False = as fast as possible (for tests)
        self._t = 0.0
        self._x = 0.0

    def _frame(self, x: float, height: float) -> BodyState:
        s = BodyState(
            x=x + random.uniform(-0.01, 0.01),          # sensor noise
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
            h = STAND_HEIGHT + JUMP_PEAK * math.sin(math.pi * p)  # arc up and down
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
