"""Verdrahtung: Quelle -> GestureDetector -> KeySender (+ optionales Overlay).

Das Mapping abstrakter Detector-Events auf konkrete Tasten passiert hier (nicht
im Detector), damit die Erkennungslogik tastenneutral und testbar bleibt.
"""

import config


def _dispatch(event: str, sender) -> str | None:
    """Setzt ein Detector-Event in einen Tastendruck um. Gibt die Taste fuer die
    Anzeige zurueck (oder None)."""
    keys = config.KEYS
    if event == "tap_left":
        sender.tap(keys["left"]);   return keys["left"]
    if event == "tap_right":
        sender.tap(keys["right"]);  return keys["right"]
    if event == "tap_up":
        sender.tap(keys["up"]);     return keys["up"]
    if event == "hold_down_start":
        sender.hold(keys["down"], True);  return keys["down"]
    if event == "hold_down_end":
        sender.hold(keys["down"], False); return None
    return None


# Event -> Zone (fuer Overlay-Hervorhebung)
_EVENT_ZONE = {
    "tap_left": "left", "tap_right": "right", "tap_up": "up",
    "hold_down_start": "down",
}


def run(source, detector, sender, overlay=None) -> None:
    was_calibrated = False
    try:
        for s in source.frames():
            events = detector.update(s)

            if detector.calibrated and not was_calibrated:
                was_calibrated = True
                print(f"Kalibriert. Baseline: "
                      + ", ".join(f"{z}={detector.baseline[z]:.2f}"
                                  for z in detector.baseline) + "\n")

            current_key = None
            triggered_zone = None
            for event in events:
                key = _dispatch(event, sender)
                if key is not None:
                    current_key = key
                triggered_zone = _EVENT_ZONE.get(event, triggered_zone)
                print(f"[{s.t:6.2f}s] {event}")

            if overlay is not None:
                action = overlay.show(s, detector, current_key, triggered_zone)
                if action in ("q", "esc"):
                    break
                elif action == "c":
                    detector.start_recalibration()
                    source.recalibrate()
                    print(">> Neu-Kalibrierung gestartet...")
    except KeyboardInterrupt:
        pass
    finally:
        sender.release_all()
        source.close()
        if overlay is not None:
            overlay.close()
    print("\nFertig.")
