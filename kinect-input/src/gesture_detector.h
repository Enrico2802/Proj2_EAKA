#pragma once

#include <vector>

#include "body_state.h"

// Erkannte Gesten.
//
// C++-Hinweis: Statt der Python-Strings ("jump", ...) nutzt man in C++ ein
// enum class - Tippfehler fallen damit schon beim Kompilieren auf statt erst
// zur Laufzeit. to_string() liefert für die Ausgabe dieselben Namen wie Python.
enum class Gesture {
    Jump,         // Körperhöhe steigt kurz über die Baseline
    CrouchStart,  // Körperhöhe sinkt deutlich unter die Baseline
    CrouchEnd,    // Person steht wieder auf
    LaneLeft,     // Person macht einen Schritt nach links
    LaneRight,    // Person macht einen Schritt nach rechts
};

const char* to_string(Gesture g);

// Regelbasierte Gestenerkennung (plan.md, Option A).
//
// Die Erkennung arbeitet rein auf BodyState-Werten (x-Versatz + Körperhöhe)
// und ist damit unabhängig davon, woher die Daten kommen (Kinect oder Mock).
//
// Beim Start kalibriert sich der Detektor selbst: die ersten Frames werden
// gemittelt und ergeben die "Ruheposition" (Baseline) der Person.
class GestureDetector {
public:
    // Pendant zu den Python-Keyword-Argumenten: ein Config-struct mit
    // Default-Werten. Aufrufer überschreiben gezielt einzelne Felder
    // (Designated Initializers, C++20):
    //
    //     GestureDetector det({.calib_frames = 10});
    struct Config {
        int    calib_frames  = 30;    // Frames für die Start-Kalibrierung (~1s bei 30 FPS)
        double jump_thresh   = 0.10;  // so viel muss die Höhe über die Baseline steigen (gleiche Einheit wie height)
        double crouch_thresh = 0.25;  // so viel muss die Höhe unter die Baseline sinken
        double lane_enter    = 0.25;  // x-Versatz, ab dem eine Seitenspur betreten gilt
        double lane_exit     = 0.15;  // x-Versatz, unter dem man wieder als "Mitte" gilt (Hysterese)
        double jump_cooldown = 0.5;   // Sekunden Sperrzeit, damit ein Sprung nicht mehrfach feuert
    };

    // C++-Eigenheit: "Config cfg = {}" als Default-Argument ist hier nicht
    // erlaubt, solange die umgebende Klasse noch unvollständig ist - deshalb
    // zwei Konstruktoren (der parameterlose delegiert an den anderen).
    GestureDetector();
    explicit GestureDetector(Config cfg);

    // Verarbeitet einen Frame und liefert die in diesem Frame erkannten Events.
    std::vector<Gesture> update(const BodyState& s);

    // C++-Hinweis: Die Python-Attribute werden hier zu Gettern - die Member
    // selbst bleiben privat (Kapselung), von außen ist nur Lesen möglich.
    bool   calibrated() const { return calibrated_; }
    double baseline_x() const { return baseline_x_; }
    double baseline_height() const { return baseline_height_; }

private:
    Config cfg_;

    // Statt der Python-Frameliste reichen laufende Summen zum Mitteln.
    int    calib_count_ = 0;
    double calib_sum_x_ = 0.0;
    double calib_sum_h_ = 0.0;

    bool   calibrated_ = false;
    double baseline_x_ = 0.0;
    double baseline_height_ = 0.0;

    int    lane_ = 0;            // -1 = links, 0 = Mitte, +1 = rechts
    bool   airborne_ = false;
    bool   crouching_ = false;
    double last_jump_t_ = -1e9;
};
