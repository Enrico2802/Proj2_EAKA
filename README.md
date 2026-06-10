# Prototyp: Kinect-Steuerung → Tastatur

Erster Prototyp für die Steuerung des Subway-Surfer-Projekts (siehe `../plan.md`).
Die Software erkennt Gesten (Springen, Ducken, Schritt links/rechts) und gibt sie
als **echte Tastendrücke** weiter — aus Sicht des Spiels sieht das aus wie eine
normale Tastatur. Das Spiel braucht dadurch **keinerlei Kinect-Anbindung**.

## Architektur

```text
Quelle                     Erkennung              Ausgabe
─────────────────────      ────────────────       ──────────────────────
MockSource (Simulation)                           SendInput() Windows-API
ManualSource (w/a/s/d)  →  GestureDetector  →     space / a / d / ctrl
später: Kinect v2          (regelbasiert,
(libfreenect2)              Baseline + Hysterese)
```

Jede Quelle liefert pro Frame nur einen `BodyState` (x-Versatz + Körperhöhe).
Die Kinect wird später als weitere Quelle eingesteckt — Erkennung und
Tastatur-Ausgabe bleiben unverändert.

## Gesten → Tasten

| Geste                  | Erkennung                              | Taste           |
| ---------------------- | -------------------------------------- | --------------- |
| Springen               | Körperhöhe kurz > Baseline + 10 cm     | Leertaste (Tap) |
| Ducken                 | Körperhöhe < Baseline − 25 cm          | Strg (gehalten) |
| Spur links/rechts      | x-Versatz über ±0.25 (mit Hysterese)   | A / D (Tap)     |

Beim Start kalibriert sich der Detektor ~1 Sekunde auf die Ruheposition der Person.
Ein Cooldown (0,5 s) verhindert, dass ein Sprung mehrfach feuert.

## Ausführen

Benötigt nur Python 3 (keine Pakete installieren).

```powershell
cd C:\Users\Kiname\Proj2_EAKA\prototyp

# 1) Simulation ansehen, ohne Tasten zu senden (Dry-Run):
python main.py

# 2) ECHTE Tasten senden: Notepad öffnen, Befehl starten, in 3s Notepad fokussieren.
#    Bei jedem simulierten Sprung erscheint ein Leerzeichen in Notepad:
python main.py --send

# 3) Interaktiv: Person selbst steuern (w=springen, s=ducken, a/d=Schritt, q=Ende):
python main.py --source manual

# Tests:
python -m unittest -v
```

## Nächste Schritte (Weg zur echten Kinect)

1. `libfreenect2` + `Protonect` zum Laufen bringen (USB 3.0, Treiber).
2. Neue Quelle `Freenect2Source` schreiben: Depth-Frame → Person segmentieren
   (nächster zusammenhängender Tiefenbereich) → Schwerpunkt-x und höchster
   Punkt → fertig ist der `BodyState`. Alles andere bleibt gleich.
3. Optional Portierung nach C++ (die Detektor-Logik ist bewusst simpel gehalten
   und lässt sich 1:1 übertragen); alternativ Python direkt an libfreenect2
   anbinden.
4. Schwellwerte (`GestureDetector`-Parameter) mit echten Personen tunen.
