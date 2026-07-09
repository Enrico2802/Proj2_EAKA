"""Kinect control prototype: gestures -> keyboard.

Pipeline (identical to the target architecture in plan.md):

    source (mock / later Kinect)  ->  GestureDetector  ->  KeySender (SendInput)

Examples:
    python main.py                          # scripted mock, logging only (dry run)
    python main.py --send                   # scripted mock, sends REAL keys (3s to focus the target window)
    python main.py --source manual          # drive the person interactively via w/a/s/d (dry run)
"""

import argparse
import time

from gesture_detector import GestureDetector
from key_sender import KeySender
from sources.mock_source import MockSource
from sources.manual_source import ManualSource

# Gesture -> key mapping (must match the game's key bindings)
ACTIONS = {
    "jump": ("tap", "space"),
    "lane_left": ("tap", "a"),
    "lane_right": ("tap", "d"),
    "crouch_start": ("press", "ctrl"),
    "crouch_end": ("release", "ctrl"),
}


def main() -> None:
    parser = argparse.ArgumentParser(description="Kinect-Gesten -> Tastatur Prototyp")
    parser.add_argument("--source", choices=["mock", "manual"], default="mock",
                        help="Datenquelle: mock = Drehbuch-Simulation, manual = per w/a/s/d steuern")
    parser.add_argument("--send", action="store_true",
                        help="Tasten WIRKLICH senden (Standard: Dry-Run, nur Logging)")
    args = parser.parse_args()

    sender = KeySender(dry_run=not args.send)
    detector = GestureDetector()
    source = MockSource() if args.source == "mock" else ManualSource()

    if args.send:
        print("ECHTER Tastatur-Modus! Fokussiere jetzt das Zielfenster (z.B. Notepad oder das Spiel)...")
        for i in (3, 2, 1):
            print(f"  Start in {i}...")
            time.sleep(1)

    print("Kalibrierung läuft - bitte still stehen...")
    was_calibrated = False

    try:
        for state in source.frames():
            events = detector.update(state)

            if detector.calibrated and not was_calibrated:
                was_calibrated = True
                print(f"Kalibriert: Baseline-Höhe = {detector.baseline_height:.2f}, "
                      f"Baseline-x = {detector.baseline_x:+.2f}\n")

            for event in events:
                kind, key = ACTIONS[event]
                print(f"[{state.t:6.2f}s] GESTE: {event:<13} -> {kind} '{key}'")
                getattr(sender, kind)(key)
    except KeyboardInterrupt:
        pass
    finally:
        # Safety net: never leave Ctrl held down
        sender.release("ctrl")

    print("\nFertig.")


if __name__ == "__main__":
    main()
