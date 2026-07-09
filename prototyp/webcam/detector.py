"""Rule-based gesture detection on ZONE RATIOS (concept document, section 6).

Works purely on the zones dict of a ZoneActivity (ratios 0..1) and is
therefore independent of where the data comes from (webcam / mock / manual).
Emits abstract events - the mapping event -> key happens in the pipeline.

Events:
    "tap_left", "tap_right", "tap_up"   - short key presses (rising edge + cooldown)
    "hold_down_start", "hold_down_end"  - hold on/off (stateful)

On startup the detector calibrates itself: the first frames are averaged
per zone and subtracted as baseline (background noise).
"""

from zones import ZoneActivity, ZONE_NAMES
import config


class GestureDetector:
    def __init__(
        self,
        calib_frames: int = config.CALIB_FRAMES,
        enter_ratio: float = config.ZONE_ENTER_RATIO,
        exit_ratio: float = config.ZONE_EXIT_RATIO,
        cooldown_s: float = config.COOLDOWN_S,
        tap_zones=("left", "right", "up"),
        hold_zones=("down",),
    ):
        self.calib_frames = calib_frames
        self.enter_ratio = enter_ratio
        self.exit_ratio = exit_ratio
        self.cooldown_s = cooldown_s
        self.tap_zones = tuple(tap_zones)
        self.hold_zones = tuple(hold_zones)

        self._calib_buffer: list[dict] = []
        self.baseline = {z: 0.0 for z in ZONE_NAMES}
        self.calibrated = False

        self._active = {z: False for z in ZONE_NAMES}     # hysteresis state per zone
        self._last_tap_t = {z: -1e9 for z in self.tap_zones}

    def start_recalibration(self) -> None:
        """Relearn the background/idle baseline (e.g. via the 'c' key)."""
        self.calibrated = False
        self._calib_buffer = []

    def _effective(self, s: ZoneActivity) -> dict:
        """Ratio minus baseline, never negative."""
        return {z: max(0.0, s.zones.get(z, 0.0) - self.baseline[z]) for z in ZONE_NAMES}

    def update(self, s: ZoneActivity) -> list[str]:
        """Processes one frame and returns the events triggered in this frame."""
        if not self.calibrated:
            self._calib_buffer.append(dict(s.zones))
            if len(self._calib_buffer) >= self.calib_frames:
                n = len(self._calib_buffer)
                for z in ZONE_NAMES:
                    self.baseline[z] = sum(b.get(z, 0.0) for b in self._calib_buffer) / n
                self.calibrated = True
                self._calib_buffer.clear()
            return []

        eff = self._effective(s)
        events: list[str] = []

        # Update the hysteresis state per zone and record edges.
        rising = {z: False for z in ZONE_NAMES}    # inactive -> active in THIS frame
        falling = {z: False for z in ZONE_NAMES}   # active -> inactive in THIS frame
        for z in ZONE_NAMES:
            was = self._active[z]
            if not was and eff[z] > self.enter_ratio:
                self._active[z] = True
                rising[z] = True
            elif was and eff[z] < self.exit_ratio:
                self._active[z] = False
                falling[z] = True
            # between exit and enter: state persists (hysteresis -> no flicker)

        # Tap zones fire once per rising edge; the strongest zone wins conflicts.
        rising_taps = [z for z in self.tap_zones if rising[z]]
        if rising_taps:
            winner = max(rising_taps, key=lambda z: eff[z])
            if s.t - self._last_tap_t[winner] >= self.cooldown_s:
                events.append("tap_" + winner)
                self._last_tap_t[winner] = s.t
            # Loser edges are deliberately discarded: their active state stays
            # set so they do not re-fire in the following frame.

        # Hold zones start on rising edges and end on falling edges.
        for z in self.hold_zones:
            if rising[z]:
                events.append("hold_" + z + "_start")
            elif falling[z]:
                events.append("hold_" + z + "_end")

        return events
