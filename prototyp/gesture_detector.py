"""Regelbasierte Gestenerkennung (plan.md, Option A).

Die Erkennung arbeitet rein auf BodyState-Werten (x-Versatz + Körperhöhe)
und ist damit unabhängig davon, woher die Daten kommen (Kinect oder Mock).

Erkannte Events:
    "jump"          - Körperhöhe steigt kurz über die Baseline
    "crouch_start"  - Körperhöhe sinkt deutlich unter die Baseline
    "crouch_end"    - Person steht wieder auf
    "lane_left"     - Person macht einen Schritt nach links
    "lane_right"    - Person macht einen Schritt nach rechts

Beim Start kalibriert sich der Detektor selbst: die ersten Frames werden
gemittelt und ergeben die "Ruheposition" (Baseline) der Person.
"""

from body_state import BodyState


class GestureDetector:
    def __init__(
        self,
        calib_frames: int = 30,    # Frames für die Start-Kalibrierung (~1s bei 30 FPS)
        jump_thresh: float = 0.10,  # so viel muss die Höhe über die Baseline steigen (gleiche Einheit wie height)
        crouch_thresh: float = 0.25,  # so viel muss die Höhe unter die Baseline sinken
        lane_enter: float = 0.25,   # x-Versatz, ab dem eine Seitenspur betreten gilt
        lane_exit: float = 0.15,    # x-Versatz, unter dem man wieder als "Mitte" gilt (Hysterese)
        jump_cooldown: float = 0.5,  # Sekunden Sperrzeit, damit ein Sprung nicht mehrfach feuert
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

        self._lane = 0           # -1 = links, 0 = Mitte, +1 = rechts
        self._airborne = False
        self._crouching = False
        self._last_jump_t = -1e9

    def update(self, s: BodyState) -> list[str]:
        """Verarbeitet einen Frame und liefert die in diesem Frame erkannten Events."""
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

        # --- Springen: steigende Flanke der Körperhöhe + Cooldown ---
        if not self._airborne and rel_h > self.jump_thresh:
            if s.t - self._last_jump_t >= self.jump_cooldown:
                events.append("jump")
                self._last_jump_t = s.t
            self._airborne = True
        elif self._airborne and rel_h < self.jump_thresh * 0.5:
            self._airborne = False  # gelandet

        # --- Ducken: zustandsbasiert mit Hysterese (Taste wird gehalten) ---
        if not self._crouching and rel_h < -self.crouch_thresh:
            self._crouching = True
            events.append("crouch_start")
        elif self._crouching and rel_h > -(self.crouch_thresh * 0.7):
            self._crouching = False
            events.append("crouch_end")

        # --- Spurwechsel: physische Position -> Spur, mit Hysterese ---
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
