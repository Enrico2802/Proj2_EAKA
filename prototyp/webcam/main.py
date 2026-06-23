"""Webcam-Bewegungssteuerung -> Tastatur. CLI-Einstieg.

Beispiele:
    python main.py                          # Mock-Drehbuch, Dry-Run (keine Tasten)
    python main.py --source manual          # w/a/s/d steuern, q=Ende
    python main.py --source webcam          # echte Kamera, Dry-Run + Overlay
    python main.py --source webcam --send   # echte Kamera + ECHTE Tasten
    python -m unittest -v                   # Unit-Tests (ohne Kamera)
"""

import argparse
import time

import config
from detector import GestureDetector
from keysender import DryRunKeySender, WinKeySender
from sources import MockSource, ManualSource
import pipeline


def _parse_grid(text: str) -> tuple[int, int]:
    cols, rows = text.lower().split("x")
    return int(cols), int(rows)


def main() -> None:
    p = argparse.ArgumentParser(description="Webcam-Gesten -> Tastatur")
    p.add_argument("--source", choices=["mock", "manual", "webcam"], default="mock")
    p.add_argument("--send", action="store_true", help="echte Tasten senden (sonst Dry-Run)")
    p.add_argument("--camera", type=int, default=config.CAMERA_INDEX)
    p.add_argument("--grid", type=str, default=None, help="Rastergroesse, z.B. 32x24")
    p.add_argument("--enter", type=float, default=config.ZONE_ENTER_RATIO)
    p.add_argument("--exit", type=float, default=config.ZONE_EXIT_RATIO)
    p.add_argument("--show", dest="show", action="store_true", default=None)
    p.add_argument("--no-show", dest="show", action="store_false")
    p.add_argument("--mirror", dest="mirror", action="store_true", default=config.MIRROR)
    p.add_argument("--no-mirror", dest="mirror", action="store_false")
    args = p.parse_args()

    if args.grid:
        config.GRID_COLS, config.GRID_ROWS = _parse_grid(args.grid)

    detector = GestureDetector(enter_ratio=args.enter, exit_ratio=args.exit)
    sender = WinKeySender() if args.send else DryRunKeySender()

    overlay = None
    if args.source == "webcam":
        from webcam_source import WebcamGridSource
        source = WebcamGridSource(camera_index=args.camera, mirror=args.mirror)
        show = True if args.show is None else args.show
        if show:
            from overlay import Overlay
            overlay = Overlay()
    elif args.source == "manual":
        source = ManualSource()
    else:
        source = MockSource()

    if args.send:
        print("ECHTER Tastatur-Modus! Fokussiere jetzt das Zielfenster "
              "(z.B. Notepad oder das Spiel)...")
        for i in (3, 2, 1):
            print(f"  Start in {i}...")
            time.sleep(1)

    print("Kalibrierung laeuft - bitte still stehen / Ruhe halten...")
    pipeline.run(source, detector, sender, overlay)


if __name__ == "__main__":
    main()
