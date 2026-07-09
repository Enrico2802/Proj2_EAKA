"""Shared data model: the body state per frame.

Whether the data comes from the Kinect (depth frame -> segmentation ->
centroid) or from a simulation - gesture detection only ever sees this.
"""

from dataclasses import dataclass


@dataclass
class BodyState:
    x: float       # horizontal offset of the person, normalized: -1.0 (far left) .. +1.0 (far right)
    height: float  # body height (highest point of the person), e.g. in meters
    t: float       # timestamp in seconds
