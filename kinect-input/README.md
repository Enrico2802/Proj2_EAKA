# kinect-input — Kinect-Steuerung (C++)

Tastatur-Emulation über Körpergesten für das Subway-Surfer-artige Uni-Spiel.
Die Software erkennt Gesten und sendet echte Tastendrücke über die Windows-API
`SendInput()` (mit Scancodes) — das Spiel braucht keine Kinect-Anbindung.

Pipeline:
```
Quelle (Mock / manuell / später Kinect)  ->  GestureDetector  ->  KeySender (SendInput)
```

## Stand

| Was | Status |
| --- | --- |
| Python-Prototyp (`../prototyp/`) | fertig, Referenz-Implementierung |
| C++-Port (`kinect-input/`) | fertig, verifiziert |
| 8 Unit-Tests (GestureDetector) | grün |
| Dry-Run-Simulation | jede Geste genau einmal erkannt |
| `--send` in Notepad | selbst testen (s.u.) |
| VS Code Build/Debug (F5, Strg+Shift+B) | eingerichtet |
| **libfreenect2 + Kinect-Anbindung** | **noch offen** |

## Nächster Schritt: Kinect v2 anbinden

1. `libfreenect2` bauen/installieren (USB 3.0 direkt am Mainboard, keine VM)
2. Mit `Protonect` prüfen, ob die Kinect Depth-Frames liefert
3. `Freenect2Source` in `src/sources/` implementieren:
   Depth-Frame → Person segmentieren (nächster zusammenhängender Tiefenbereich)
   → Schwerpunkt-x normieren + höchsten Punkt in Meter → `BodyState`
4. `GestureDetector` und `KeySender` bleiben **unverändert**
5. Schwellwerte mit echten Personen tunen (ggf. als CLI-Parameter herausziehen)

## Struktur

```
kinect-input/
  src/
    body_state.h                  # Datenmodell (x, height, t)
    gesture_detector.h / .cpp     # regelbasierte Gestenerkennung
    key_sender.h / .cpp           # SendInput() mit Scancodes, Dry-Run-Modus
    sources/
      source.h                    # gemeinsames Interface (next())
      mock_source.h / .cpp        # simuliertes Drehbuch (ohne Kinect testbar)
      manual_source.h / .cpp      # w/a/s/d-Steuerung über Konsole
    main.cpp                      # CLI-Einstieg
  tests/
    test_gesture_detector.cpp     # 8 Unit-Tests, kein Framework
  CMakeLists.txt
```

Pendant zur Python-Referenz:

| Python (`../prototyp/`)     | C++                                       |
| --------------------------- | ----------------------------------------- |
| `body_state.py`             | `src/body_state.h`                        |
| `gesture_detector.py`       | `src/gesture_detector.h` / `.cpp`         |
| `key_sender.py`             | `src/key_sender.h` / `.cpp`              |
| Duck-Typing                 | `src/sources/source.h` (Interface)        |
| `sources/mock_source.py`    | `src/sources/mock_source.h` / `.cpp`      |
| `sources/manual_source.py`  | `src/sources/manual_source.h` / `.cpp`    |
| `main.py`                   | `src/main.cpp`                            |
| `test_gesture_detector.py`  | `tests/test_gesture_detector.cpp`         |

## Bauen

Toolchain: MSYS2 UCRT64 — `g++`, `cmake`, `ninja` unter `C:\msys64\ucrt64\bin`.

**In VS Code:** `Strg+Shift+B` (Build-Task) oder `F5` (bauen + debuggen).

**Im Terminal** (nach Neustart, wenn `C:\msys64\ucrt64\bin` im PATH):
```
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build
```

## Ausführen

```powershell
.\build\kinect-input.exe                  # Mock-Drehbuch, nur Logging (Dry-Run)
.\build\kinect-input.exe --source manual  # Person per w/a/s/d steuern, q = Ende
.\build\kinect-input.exe --send           # ECHTE Tasten senden — im 3s-Countdown Notepad fokussieren
```

## Tests

```powershell
ctest --test-dir build --output-on-failure
# oder direkt:
.\build\test_gesture_detector.exe
```

## Tastenbelegung (Team-Absprache)

| Geste             | Erkennung                                    | Taste           |
| ----------------- | -------------------------------------------- | --------------- |
| Springen          | Körperhöhe kurz > Baseline + 10 cm           | Leertaste (Tap) |
| Ducken            | Körperhöhe < Baseline − 25 cm               | Strg (halten)   |
| Spur links/rechts | x-Versatz über ±0.25 (Hysterese: raus 0.15) | A / D (Tap)     |
