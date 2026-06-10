#pragma once

#include "sources/source.h"

// Manuelle Quelle: simulierte Person per Konsolen-Tasten steuern.
//
// Praktisch zum interaktiven Ausprobieren der Gestenerkennung ohne Kinect:
//
//     w = springen      s = ducken an/aus
//     a = Schritt links d = Schritt rechts
//     q = beenden
//
// Hinweis: Die Tasten werden im KONSOLENFENSTER gelesen. Diesen Modus daher
// im Dry-Run verwenden (sonst landen die gesendeten Tasten ebenfalls in
// der Konsole, weil sie den Fokus hat).
class ManualSource : public Source {
public:
    bool next(BodyState& out) override;

private:
    static constexpr int    kFps = 30;
    static constexpr double kDt = 1.0 / kFps;
    static constexpr double kStandHeight  = 1.75;
    static constexpr double kJumpPeak     = 0.30;
    static constexpr double kCrouchHeight = 1.30;
    static constexpr double kSideX        = 0.55;

    bool   first_call_ = true;
    double x_ = 0.0;
    bool   crouching_ = false;
    double jump_t_ = -1.0;  // Zeitpunkt des Absprungs, -1 = nicht in der Luft
    double t_ = 0.0;
};
