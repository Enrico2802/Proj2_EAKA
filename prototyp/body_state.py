"""Gemeinsames Datenmodell: der Körperzustand pro Frame.

Egal ob die Daten aus der Kinect (Depth-Frame -> Segmentierung -> Schwerpunkt)
oder aus einer Simulation kommen - die Gestenerkennung sieht immer nur das hier.
"""

from dataclasses import dataclass


@dataclass
class BodyState:
    x: float       # horizontaler Versatz der Person, normiert: -1.0 (ganz links) .. +1.0 (ganz rechts)
    height: float  # Körperhöhe (höchster Punkt der Person), z.B. in Metern
    t: float       # Zeitstempel in Sekunden
