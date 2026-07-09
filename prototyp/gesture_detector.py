"""Rule-based gesture detection (plan.md, option A).

Detection works purely on BodyState values (x offset + body height) and is
therefore independent of where the data comes from (Kinect or mock).

Detected events:
    "jump"          - body height briefly rises above the baseline
    "crouch_start"  - body height drops clearly below the baseline
    "crouch_end"    - the person stands up again
    "lane_left"     - the person takes a step to the left
    "lane_right"    - the person takes a step to the right

On startup the detector calibrates itself: the first frames are averaged
and yield the person's "idle position" (baseline).
"""

from body_state import BodyState


class GestureDetector:
    def __init__(
        self,
        calib_frames: int = 30,    # frames for the startup calibration (~1s at 30 FPS)
        jump_thresh: float = 0.10,  # how far the height must rise above the baseline (same unit as height)
        crouch_thresh: float = 0.25,  # how far the height must drop below the baseline
        lane_enter: float = 0.25,   # x offset at which a side lane counts as entered
        lane_exit: float = 0.15,    # x offset below which one counts as "center" again (hysteresis)
        jump_cooldown: float = 0.5,  # lockout in seconds so a jump does not fire multiple times
    ):
        self.calib_frames = calib_frames
        self.jump_thresh = jump_thresh
        self.crouch_thresh = crouch_thresh
        self.lane_enter = lane_enter
        self.lane_exit = lane_exit
        self.jump_cooldown = jump_cooldown

        self._calib_buffer: list[BodyState] = []
        self.baseline_x = 0.0
        self.baseline_height = 0.0
        self.calibrated = False

        self._lane = 0           # -1 = left, 0 = center, +1 = right
        self._airborne = False
        self._crouching = False
        self._last_jump_t = -1e9

    def update(self, s: BodyState) -> list[str]:
        """Processes one frame and returns the events detected in this frame."""
        if not self.calibrated:
            self._calib_buffer.append(s)
            if len(self._calib_buffer) >= self.calib_frames:
                n = len(self._calib_buffer)
                self.baseline_x = sum(b.x for b in self._calib_buffer) / n
                self.baseline_height = sum(b.height for b in self._calib_buffer) / n
                self.calibrated = True
                self._calib_buffer.clear()
            return []

        events: list[str] = []
        rel_x = s.x - self.baseline_x
        rel_h = s.height - self.baseline_height

        # Jump detection with cooldown.
        if not self._airborne and rel_h > self.jump_thresh:
            if s.t - self._last_jump_t >= self.jump_cooldown:
                events.append("jump")
                self._last_jump_t = s.t
            self._airborne = True
        elif self._airborne and rel_h < self.jump_thresh * 0.5:
            self._airborne = False

        # Crouch detection with hysteresis.
        if not self._crouching and rel_h < -self.crouch_thresh:
            self._crouching = True
            events.append("crouch_start")
        elif self._crouching and rel_h > -(self.crouch_thresh * 0.7):
            self._crouching = False
            events.append("crouch_end")

        # Lane changes use hysteresis to avoid flicker near the center.
        target_lane = self._lane
        if self._lane == 0:
            if rel_x < -self.lane_enter:
                target_lane = -1
            elif rel_x > self.lane_enter:
                target_lane = +1
        elif self._lane == -1 and rel_x > -self.lane_exit:
            target_lane = 0
        elif self._lane == +1 and rel_x < self.lane_exit:
            target_lane = 0

        if target_lane < self._lane:
            events.append("lane_left")
        elif target_lane > self._lane:
            events.append("lane_right")
        self._lane = target_lane

        return events
