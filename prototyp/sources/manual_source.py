"""Manuelle Quelle: simulierte Person per Konsolen-Tasten steuern.

Praktisch zum interaktiven Ausprobieren der Gestenerkennung ohne Kinect:

    w = springen      s = ducken an/aus
    a = Schritt links d = Schritt rechts
    q = beenden

Hinweis: Die Tasten werden im KONSOLENFENSTER gelesen. Diesen Modus daher
mit --dry-run verwenden (sonst landen die gesendeten Tasten ebenfalls in
der Konsole, weil sie den Fokus hat).
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
        jump_t = -1.0  # Zeitpunkt des Absprungs, -1 = nicht in der Luft
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
                p = (t - jump_t) / 0.45  # Sprungdauer 0.45s
                if p >= 1.0:
                    jump_t = -1.0
                else:
                    height = STAND_HEIGHT + JUMP_PEAK * math.sin(math.pi * p)

            yield BodyState(x=x, height=height, t=t)
            t += DT
            time.sleep(DT)
