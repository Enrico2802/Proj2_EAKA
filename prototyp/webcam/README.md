# Webcam-Bewegungssteuerung → Tastatur

Webcam statt Kinect: Eine normale Webcam, MOG2-Hintergrundsubtraktion und ein
Bewegungs-Raster (Motion-Energy-Grid) erkennen Gesten in vier Zonen und senden
echte Tastendrücke über `SendInput()` (Scancodes). Kein ML-Modell nötig.

Pipeline:
```
Webcam → WebcamGridSource (MOG2 + Grid + Zonen) → GestureDetector → KeySender
                                                          └→ Overlay (Monitor 2)
```

## Gesten → Tasten (Default, in `config.py` umbelegbar)

| Zone            | Geste                | Taste            | Art  |
| --------------- | -------------------- | ---------------- | ---- |
| oben Mitte      | Arme hoch            | Leertaste        | Tap  |
| links           | Arm in linke Ecke    | A                | Tap  |
| rechts          | Arm in rechte Ecke   | D                | Tap  |
| unteres Band    | Ducken/Hocken        | S                | Hold |

Schwellwert ist **anteilig** (15 % aktive Zellen rein / 8 % raus, Hysterese),
also rasterunabhängig. Cooldown 0,5 s gegen Doppelauslösung. Start-Kalibrierung
~1 s in Ruhe.

## Setup

```powershell
cd prototyp\webcam
pip install -r requirements.txt
```

## Bedienung

```powershell
python main.py                          # Mock-Drehbuch, Dry-Run (keine Tasten)
python main.py --source manual          # w/a/s/d steuern, q=Ende
python main.py --source webcam          # echte Kamera, Dry-Run + Overlay
python main.py --source webcam --send   # echte Kamera + ECHTE Tasten
python -m unittest -v                   # 9 Unit-Tests (ohne Kamera)
```

Flags: `--camera N`, `--grid 32x24`, `--enter 0.15`, `--exit 0.08`,
`--show`/`--no-show`, `--mirror`/`--no-mirror`.

Im Overlay-Fenster: `m` = Bild/Maske umschalten, `c` = neu kalibrieren,
`q`/`ESC` = beenden (löst gehaltene Tasten als Not-Aus).

## Dateien

| Datei | Zweck |
| --- | --- |
| `config.py` | alle Parameter (Zonen, Schwellen, Tasten, Kamera) |
| `zones.py` | Daten-Vertrag `ZoneActivity` (Zonen-Anteile 0..1) |
| `detector.py` | Gestenerkennung (Hysterese, Cooldown, Tap/Hold) — hardwarefrei |
| `test_detector.py` | 9 Unit-Tests der Erkennungslogik |
| `sources.py` | `MockSource`, `ManualSource` |
| `webcam_source.py` | `WebcamGridSource` (MOG2 → Grid → Zonen) |
| `keysender.py` | `DryRunKeySender`, `WinKeySender` (SendInput/Scancodes) |
| `overlay.py` | Beweis-Screen auf Monitor 2 |
| `pipeline.py` | verdrahtet Quelle → Detector → KeySender (+ Overlay) |
| `main.py` | CLI |

## Stand

| Schritt | Status |
| --- | --- |
| 1 Gerüst + config | fertig |
| 2 Detector + 9 Tests | fertig, grün |
| 3 Mock + DryRun + Pipeline | fertig, verifiziert |
| 4 ManualSource | fertig (interaktiv testen) |
| 5–8 WebcamGridSource | fertig (an echter Kamera testen) |
| 9 Overlay | fertig (an Kamera/Monitor 2 testen) |
| 10 WinKeySender (`--send`) | fertig |
