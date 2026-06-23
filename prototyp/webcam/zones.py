"""Der EINE Daten-Vertrag zwischen Quelle und Detector.

Eine Quelle (Mock / Manual / Webcam) liefert pro Frame ein ZoneActivity mit den
Bewegungs-ANTEILEN je Zone (0.0 .. 1.0). Der GestureDetector liest ausschliesslich
.zones und .t und ist damit ohne Kamera testbar und 1:1 nach Rust portierbar.
frame/grid sind optional und nur fuer das Overlay gedacht.
"""

from dataclasses import dataclass, field

ZONE_NAMES = ("left", "right", "up", "down")


@dataclass
class ZoneActivity:
    zones: dict            # {"left":float,"right":float,"up":float,"down":float}, je 0..1
    t: float               # Zeitstempel in Sekunden (monoton steigend)
    frame: object = None   # optional: BGR-Bild ODER Maske (nur Overlay)
    grid: object = None    # optional: 2D-bool/uint8-Array aktiver Zellen (nur Overlay)


def full_zones(partial: dict) -> dict:
    """Fuellt ein Teil-Dict zu allen vier Zonen auf (fehlende -> 0.0)."""
    return {name: float(partial.get(name, 0.0)) for name in ZONE_NAMES}
