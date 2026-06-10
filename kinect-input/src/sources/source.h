#pragma once

#include "body_state.h"

// Gemeinsame Schnittstelle aller Datenquellen (Mock, manuell, später Kinect).
//
// C++-Hinweis: Python-Generatoren (yield) gibt es in C++ so nicht - stattdessen
// das Pull-Prinzip: next() füllt den nächsten Frame in `out` und liefert false,
// wenn die Quelle zu Ende ist. Die abstrakte Basisklasse mit virtuellen
// Methoden ersetzt das Duck-Typing aus Python: main.cpp arbeitet nur mit
// Source* und kennt die konkrete Quelle nicht.
class Source {
public:
    virtual ~Source() = default;  // virtueller Destruktor: Pflicht bei Vererbung über Basisklassen-Pointer

    virtual bool next(BodyState& out) = 0;  // = 0: rein virtuell, muss überschrieben werden
};
