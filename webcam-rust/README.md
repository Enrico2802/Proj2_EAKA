# Webcam-Steuerung → Tastatur (Rust-Port)

Rust-Portierung des Python-Prototyps (`prototyp/webcam/`). Gleiche Pipeline,
gleiche anteiligen Schwellwerte, gleiche 9 Tests. Webcam → MOG2-Bewegungsmaske →
32×24-Raster → Zonen-Anteile → Gesten → echte Tastendrücke (Win32 `SendInput`).

```
Webcam → WebcamGridSource (MOG2+Grid+Zonen) → GestureDetector → KeySender
                                                      └→ Overlay (minifb, Monitor 2)
```

## Toolchain (MSYS2 UCRT64)

Einmalig installieren:

```bash
pacman -S mingw-w64-ucrt-x86_64-opencv \
          mingw-w64-ucrt-x86_64-clang \
          mingw-w64-ucrt-x86_64-rust
```

Wichtig:
- **UCRT64-Rust** (GNU-ABI), nicht MSVC-rustup — passt zur OpenCV aus MSYS2.
- Die `opencv`-Crate muss **≥ 0.98** sein (ältere unterstützen OpenCV 4.13 nicht).
- `highgui` ist bewusst deaktiviert: dessen DLL ist gegen **Qt6** gebaut und
  scheitert hier beim Laden (Qt6 → WinRT-API-Set wird nicht aufgelöst). Das
  Overlay-Fenster läuft daher über **minifb** (reines Rust, kein Qt).
- Build-Variablen (`LIBCLANG_PATH`, `PKG_CONFIG_PATH`) stehen in
  [`.cargo/config.toml`](.cargo/config.toml) — kein manuelles Exportieren nötig.

## Bauen & Ausführen

`C:\msys64\ucrt64\bin` muss zum **Ausführen** im PATH liegen (OpenCV-DLLs).
In einer MSYS2-UCRT64-Shell ist das automatisch der Fall; in PowerShell:

```powershell
$env:Path = "C:\msys64\ucrt64\bin;" + $env:Path
```

```bash
cargo test                         # 9 Detector-Tests (ohne Kamera)
cargo run                          # Mock-Drehbuch, Dry-Run
cargo run -- --source manual       # w/a/s/d steuern, q=Ende
cargo run -- --source webcam       # echte Kamera, Dry-Run + Overlay
cargo run -- --source webcam --send  # echte Kamera + ECHTE Tasten
cargo run --bin probe              # Diagnose: Kamera öffnen + Frames lesen
```

Flags: `--camera N`, `--no-show`, `--no-mirror`, `--enter 0.15`, `--exit 0.08`.

Im Overlay: `m` = Bild/Maske, `c` = neu kalibrieren, `k` = **Send an/aus**
(echte Tasten zur Laufzeit), `q`/`ESC` = Ende.

## Dateien

| Datei | Zweck |
| --- | --- |
| `src/zones.rs` | Daten-Vertrag `ZoneActivity` (Zonen-Anteile 0..1) |
| `src/detector.rs` | Gestenerkennung + 9 Unit-Tests (hardwarefrei) |
| `src/keysender.rs` | umschaltbarer Sender, Win32 `SendInput` (rohes FFI) |
| `src/sources.rs` | `MockSource`, `ManualSource` |
| `src/webcam_source.rs` | `WebcamGridSource` (OpenCV: MOG2 → Grid → Zonen) |
| `src/overlay.rs` | Beweis-Screen (minifb + OpenCV-Zeichnung) |
| `src/pipeline.rs` | Event → Taste |
| `src/config.rs` | alle Parameter |
| `src/main.rs` | CLI |
| `src/bin/probe.rs` | Kamera-Diagnose |

## Stand

| Teil | Status |
| --- | --- |
| Toolchain (opencv 0.98 / OpenCV 4.13 / UCRT64) | läuft |
| Detector + 9 Tests | grün |
| Mock/Manual + Pipeline + KeySender | fertig, Mock verifiziert |
| WebcamGridSource (MOG2) | fertig (Kamera-Lesen verifiziert) |
| Overlay (minifb) | fertig (live an Kamera/Monitor 2 testen) |
