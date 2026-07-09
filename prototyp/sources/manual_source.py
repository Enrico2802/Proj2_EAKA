"""Manual source: drive the simulated person via console keys.

Handy for trying out gesture detection interactively without a Kinect:

    w = jump          s = toggle crouch
    a = step left     d = step right
    q = quit

Note: the keys are read in the CONSOLE WINDOW. Use this mode with dry run
(otherwise the sent keys also end up in the console because it has focus).
"""

import math
import msvcrt
import time

from body_state import BodyState

FPS = 30
DT = 1.0 / FPS

STAND_HEIGHT = 1.75
JUMP_PEAK = 0.30
CROUCH_HEIGHT = 1.30
SIDE_X = 0.55


class ManualSource:
    def frames(self):
        print("Steuerung: [w] springen  [s] ducken an/aus  [a]/[d] Schritt links/rechts  [q] beenden")
        x = 0.0
        crouching = False
        jump_t = -1.0  # takeoff time, -1 = not airborne
        t = 0.0
        while True:
            while msvcrt.kbhit():
                key = msvcrt.getwch().lower()
                if key == "q":
                    return
                elif key == "w" and jump_t < 0:
                    jump_t = t
                elif key == "s":
                    crouching = not crouching
                elif key == "a":
                    x = max(-SIDE_X, x - SIDE_X)
                elif key == "d":
                    x = min(+SIDE_X, x + SIDE_X)

            height = CROUCH_HEIGHT if crouching else STAND_HEIGHT
            if jump_t >= 0:
                p = (t - jump_t) / 0.45  # jump duration 0.45 s
                if p >= 1.0:
                    jump_t = -1.0
                else:
                    height = STAND_HEIGHT + JUMP_PEAK * math.sin(math.pi * p)

            yield BodyState(x=x, height=height, t=t)
            t += DT
            time.sleep(DT)
