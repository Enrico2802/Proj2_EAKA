# Webcam-Steuerung → Tastatur (Rust-Port)

Rust-Portierung des Python-Prototyps (`prototyp/webcam/`), inzwischen mit
robusterer Erkennung als der Prototyp. Webcam → MOG2-Bewegungsmaske →
32×24-Raster → Zonen-Anteile → Gesten → echte Tastendrücke (Win32 `SendInput`).

Robustheits-Mechanismen (alle Parameter in `src/config.rs`):
- **Eingefrorenes Hintergrundmodell**: nach ~2 s Warmup lernt MOG2 nicht mehr
  weiter — eine still gehaltene Pose (Ducken!) verschwindet sonst nach ~10-25 s
  aus der Maske und der Hold bricht ab. Lichtwechsel: Taste `c` oder automatisch.
- **Auto-Rekalibrierung**: ist >60 % des Rasters ~1,5 s aktiv, war das ein
  Licht-/Szenenwechsel — Hintergrund und Baseline werden neu gelernt, solange
  gehen keine Fehl-Tasten raus.
- **2-Frame-Bestätigung**: Flanken zählen erst nach 2 Frames in Folge —
  Ein-Frame-Ausreißer (Rauschen, Belichtungssprung) lösen nichts mehr aus.
- **Pro-Zone-Schwellen**: enter/exit je Zone (`[left, right, up, down]`), weil
  ein dünner Arm die up-Zone ganz anders füllt als der Körper das down-Band.
- **Optical-Flow-Richtungs-Gate** (nur Tap-Zonen): ein Tap feuert nur, wenn die
  Bewegung auch in Gestenrichtung zeigt — Durchlaufen/Vorbeugen löst nicht aus.
  Abschaltbar mit `--no-flow`.

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
cargo test                         # 14 Detector-Tests (ohne Kamera)
cargo run                          # Mock-Drehbuch, Dry-Run
cargo run -- --source manual       # w/a/s/d steuern, q=Ende
cargo run -- --source webcam       # echte Kamera, Dry-Run + Overlay
cargo run -- --source webcam --send  # echte Kamera + ECHTE Tasten
cargo run --bin probe              # Diagnose: Kamera öffnen + Frames lesen
```

Flags: `--camera N`, `--no-show`, `--no-mirror`, `--no-flow`,
`--enter 0.15` / `--exit 0.08` (überschreibt die Schwelle **aller** vier Zonen;
ohne Flag gelten die Pro-Zone-Werte aus `config.rs`).

Im Overlay: `m` = Bild/Maske, `c` = neu kalibrieren, `k` = **Send an/aus**
(echte Tasten zur Laufzeit), `q`/`ESC` = Ende.

## Dateien

| Datei | Zweck |
| --- | --- |
| `src/zones.rs` | Daten-Vertrag `ZoneActivity` (Zonen-Anteile 0..1) |
| `src/detector.rs` | Gestenerkennung + 14 Unit-Tests (hardwarefrei) |
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
| Detector + 14 Tests | grün |
| Mock/Manual + Pipeline + KeySender | fertig, Mock verifiziert |
| WebcamGridSource (MOG2 + Flow-Gate + Auto-Rekalib) | fertig (live an der Kamera testen) |
| Overlay (minifb) | fertig (live an Kamera/Monitor 2 testen) |
