# Kinect-Steuerung → Tastatur

Steuerungs-Komponente des Subway-Surfer-Uni-Projekts: Die Software erkennt
Körpergesten und sendet **echte Tastendrücke** über die Windows-API `SendInput()`
(Scancodes) — das Spiel braucht dadurch keinerlei Kinect-Anbindung.

Pipeline:
```
Quelle (Mock / manuell / später Kinect)  →  GestureDetector  →  KeySender (SendInput)
```

## Gesten → Tasten (Team-Absprache)

| Geste             | Erkennung                                    | Taste           |
| ----------------- | -------------------------------------------- | --------------- |
| Springen          | Körperhöhe kurz > Baseline + 10 cm           | Leertaste (Tap) |
| Ducken            | Körperhöhe < Baseline − 25 cm               | Strg (halten)   |
| Spur links/rechts | x-Versatz über ±0.25 (Hysterese: raus 0.15) | A / D (Tap)     |

Beim Start kalibriert sich der Detektor ~1 s auf die Ruheposition. Ein Cooldown
(0,5 s) verhindert Mehrfach-Auslösung beim Sprung.

## Implementierungen

### Python-Prototyp (`prototyp/`)

Referenz-Implementierung. Benötigt nur Python 3, keine weiteren Pakete.

```powershell
cd prototyp

python main.py                  # Mock-Drehbuch, Dry-Run
python main.py --send           # ECHTE Tasten (3s Countdown, Notepad fokussieren)
python main.py --source manual  # w/a/s/d steuern, q = Ende
python -m unittest -v           # 8 Unit-Tests
```

### C++-Port (`kinect-input/`)

Produktions-Implementierung, bereit für die Kinect-Anbindung. Gleiche Pipeline,
gleiche Schwellwerte, gleiche 8 Tests — alle grün. Toolchain: MSYS2 UCRT64.

```powershell
cd kinect-input

# Bauen (einmalig konfigurieren, danach nur noch build):
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build

.\build\kinect-input.exe                  # Mock-Drehbuch, Dry-Run
.\build\kinect-input.exe --send           # ECHTE Tasten
.\build\kinect-input.exe --source manual  # w/a/s/d steuern

ctest --test-dir build --output-on-failure  # Tests
```

In VS Code: **Strg+Shift+B** baut, **F5** startet den Debugger.

## Stand & nächste Schritte

| Was | Status |
| --- | --- |
| Python-Prototyp | fertig |
| C++-Port | fertig, verifiziert |
| VS Code Build/Debug | eingerichtet |
| **libfreenect2 + `Freenect2Source`** | **offen** |

Nächster Schritt: `libfreenect2` bauen, mit `Protonect` prüfen ob die Kinect
Depth-Frames liefert (USB 3.0 direkt am Mainboard), dann `Freenect2Source`
implementieren. `GestureDetector` und `KeySender` bleiben dabei unverändert.
Details → [`kinect-input/README.md`](kinect-input/README.md).
Testing

## Rust-Experiment

Auf dem Branch `rust-input-experiment` gibt es zusaetzlich einen experimentellen
Rust-Port in `kinect-input-rust/`. Portiert sind `GestureDetector`, Mock-/Manual-
Source, `KeySender` und die 8 Unit-Tests. Die Kinect-Anbindung bleibt dort noch
bewusst offen.

```powershell
cd kinect-input-rust
cargo test
cargo run
cargo run -- --source manual
cargo run -- --send
```
