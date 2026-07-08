# kinect-input-rust

Rust-Experiment zur frueheren Kinect-Input-Pipeline.

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
- Die Kinect-Anbindung bleibt offen. Fuer einen echten Rust-Kinect-Port braucht man spaeter entweder Bindings, einen kleinen nativen Wrapper oder eine andere Kinect-API.

## Naechste Schritte

1. Rust installieren: <https://rustup.rs/>
2. `cargo test` ausfuehren.
3. `cargo run` mit Mock pruefen.
4. Optional: `cargo run -- --send` in Notepad testen.
5. Spaeter: `Freenect2Source`-Strategie entscheiden.
