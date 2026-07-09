"""THE single data contract between source and detector.

A source (mock / manual / webcam) delivers one ZoneActivity per frame with
the motion RATIOS per zone (0.0 .. 1.0). The GestureDetector reads only
.zones and .t and is therefore testable without a camera and portable to
Rust 1:1. frame/grid are optional and only meant for the overlay.
"""

from dataclasses import dataclass, field

ZONE_NAMES = ("left", "right", "up", "down")


@dataclass
class ZoneActivity:
    zones: dict            # {"left":float,"right":float,"up":float,"down":float}, each 0..1
    t: float               # timestamp in seconds (monotonically increasing)
    frame: object = None   # optional: BGR image OR mask (overlay only)
    grid: object = None    # optional: 2D bool/uint8 array of active cells (overlay only)


def full_zones(partial: dict) -> dict:
    """Fills a partial dict up to all four zones (missing -> 0.0)."""
    return {name: float(partial.get(name, 0.0)) for name in ZONE_NAMES}
