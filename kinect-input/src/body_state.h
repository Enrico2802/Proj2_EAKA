#pragma once

// Gemeinsames Datenmodell: der Körperzustand pro Frame.
//
// Egal ob die Daten aus der Kinect (Depth-Frame -> Segmentierung -> Schwerpunkt)
// oder aus einer Simulation kommen - die Gestenerkennung sieht immer nur das hier.
//
// C++-Hinweis: Das Pendant zur Python-@dataclass ist ein einfaches struct.
// "struct" statt "class" heißt nur: alle Member sind standardmäßig public.

struct BodyState {
    double x = 0.0;       // horizontaler Versatz der Person, normiert: -1.0 (ganz links) .. +1.0 (ganz rechts)
    double height = 0.0;  // Körperhöhe (höchster Punkt der Person), z.B. in Metern
    double t = 0.0;       // Zeitstempel in Sekunden
};
