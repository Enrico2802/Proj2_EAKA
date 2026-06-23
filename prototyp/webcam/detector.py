"""Regelbasierte Gestenerkennung auf ZONEN-ANTEILEN (KONZEPT Abs. 6).

Arbeitet rein auf dem zones-Dict eines ZoneActivity (Anteile 0..1) und ist damit
unabhaengig davon, woher die Daten kommen (Webcam / Mock / Manual). Liefert
abstrakte Events - das Mapping Event -> Taste passiert in der Pipeline.

Events:
    "tap_left", "tap_right", "tap_up"   - kurze Tasten (steigende Flanke + Cooldown)
    "hold_down_start", "hold_down_end"  - Halten an/aus (zustandsbasiert)

Beim Start kalibriert sich der Detektor: die ersten Frames werden je Zone
gemittelt und als Baseline (Grundrauschen) abgezogen.
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

        self._active = {z: False for z in ZONE_NAMES}     # Hysterese-Zustand je Zone
        self._last_tap_t = {z: -1e9 for z in self.tap_zones}

    def start_recalibration(self) -> None:
        """Hintergrund-/Ruhe-Baseline neu lernen (z.B. Taste 'c')."""
        self.calibrated = False
        self._calib_buffer = []

    def _effective(self, s: ZoneActivity) -> dict:
        """Anteil minus Baseline, nie negativ."""
        return {z: max(0.0, s.zones.get(z, 0.0) - self.baseline[z]) for z in ZONE_NAMES}

    def update(self, s: ZoneActivity) -> list[str]:
        """Verarbeitet einen Frame, liefert die in diesem Frame ausgeloesten Events."""
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

        # --- 1) neuen Hysterese-Zustand je Zone bestimmen + Flanken merken ---
        rising = {z: False for z in ZONE_NAMES}    # inaktiv -> aktiv in DIESEM Frame
        falling = {z: False for z in ZONE_NAMES}   # aktiv -> inaktiv in DIESEM Frame
        for z in ZONE_NAMES:
            was = self._active[z]
            if not was and eff[z] > self.enter_ratio:
                self._active[z] = True
                rising[z] = True
            elif was and eff[z] < self.exit_ratio:
                self._active[z] = False
                falling[z] = True
            # zwischen exit und enter: Zustand bleibt (Hysterese -> kein Flackern)

        # --- 2) Tap-Zonen: bei steigender Flanke EIN Tap, Konflikt: staerkste gewinnt ---
        rising_taps = [z for z in self.tap_zones if rising[z]]
        if rising_taps:
            winner = max(rising_taps, key=lambda z: eff[z])
            if s.t - self._last_tap_t[winner] >= self.cooldown_s:
                events.append("tap_" + winner)
                self._last_tap_t[winner] = s.t
            # Verlierer-Flanken werden bewusst verworfen: ihr Aktiv-Zustand bleibt
            # gesetzt, damit sie nicht im Folgeframe nachfeuern.

        # --- 3) Hold-Zonen: Start bei steigender, Ende bei fallender Flanke ---
        for z in self.hold_zones:
            if rising[z]:
                events.append("hold_" + z + "_start")
            elif falling[z]:
                events.append("hold_" + z + "_end")

        return events
