# kinect-input-rust

Rust-Experiment zum bestehenden C++-Input-Service.

Der Port bildet dieselbe Pipeline ab:

```text
Quelle (Mock / manuell / spaeter Kinect) -> GestureDetector -> KeySender
```

## Stand

| Was | Status |
| --- | --- |
| `GestureDetector` | portiert |
| 8 Unit-Tests | portiert |
| Mock-Quelle | portiert |
| Manuelle Quelle | portiert, Eingabe mit Enter |
| `SendInput()`-KeySender | portiert, Windows-only |
| Kinect-Quelle | bewusst noch offen |

## Ausfuehren

```powershell
cd kinect-input-rust

cargo test
cargo run
cargo run -- --source manual
cargo run -- --send
```

Im manuellen Modus werden Eingaben zeilenweise gelesen:

```text
w + Enter = springen
s + Enter = ducken an/aus
a + Enter = Schritt links
d + Enter = Schritt rechts
q + Enter = Ende
```

## Warum dieser Port so gebaut ist

- Keine externen Rust-Crates: Der erste Build bleibt klein und offline-faehig.
- Die Gestenlogik ist eine Library, damit `cargo test` sie direkt testet.
- Der einzige `unsafe`-Block sitzt in `key_sender.rs`, wo die Win32-Funktionen `MapVirtualKeyW` und `SendInput` angebunden sind.
- Die Kinect-Anbindung bleibt offen, weil `libfreenect2` C++-nah ist. Fuer einen echten Rust-Kinect-Port braucht man spaeter entweder Bindings, einen kleinen C++-Shim oder eine andere Kinect-API.

## Vergleich zum C++-Port

| C++ | Rust |
| --- | --- |
| `body_state.h` | `src/body_state.rs` |
| `gesture_detector.h/.cpp` | `src/gesture_detector.rs` |
| `key_sender.h/.cpp` | `src/key_sender.rs` |
| `Source` Interface | `sources::Source` Trait |
| `MockSource` | `sources::MockSource` |
| `ManualSource` | `sources::ManualSource` |
| `main.cpp` | `src/main.rs` |

## Naechste Schritte

1. Rust installieren: <https://rustup.rs/>
2. `cargo test` ausfuehren.
3. `cargo run` mit Mock pruefen.
4. Optional: `cargo run -- --send` in Notepad testen.
5. Spaeter: `Freenect2Source`-Strategie entscheiden.
