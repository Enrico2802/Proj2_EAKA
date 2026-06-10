#pragma once

#include <cstddef>
#include <random>
#include <string>
#include <vector>

#include "sources/source.h"

// Simulierte Person vor der Kinect.
//
// Spielt ein festes "Drehbuch" ab: stehen (Kalibrierung), springen, Schritt
// nach links, zurück, ducken, Schritt nach rechts, zurück, springen.
// So lässt sich die komplette Pipeline ohne Hardware testen und vorführen.
//
// Später wird diese Klasse durch eine Freenect2Source ersetzt, die dieselben
// BodyState-Frames aus echten Kinect-Depth-Daten berechnet.
//
// C++-Hinweis: Statt des Python-Generators wird das komplette Drehbuch im
// Konstruktor vorberechnet (inkl. Sensor-Rauschen); next() spielt es dann
// Frame für Frame ab.
class MockSource : public Source {
public:
    explicit MockSource(bool realtime = true);  // false = so schnell wie möglich (für Tests)

    bool next(BodyState& out) override;

private:
    struct Step {
        std::string message;  // wird vor diesem Frame ausgegeben (leer = nichts)
        double x = 0.0;
        double height = 0.0;
    };

    static constexpr int    kFps = 30;
    static constexpr double kDt = 1.0 / kFps;
    static constexpr double kStandHeight  = 1.75;  // m - Körpergröße der simulierten Person
    static constexpr double kJumpPeak     = 0.30;  // m - wie hoch der höchste Punkt beim Sprung steigt
    static constexpr double kCrouchHeight = 1.30;  // m - Körperhöhe in der Hocke
    static constexpr double kSideX        = 0.55;  // normierter x-Versatz einer Seitenspur

    double noise();
    void say(std::string message);
    void push(double x, double height);
    void hold(double seconds, double height = kStandHeight);
    void jump(double duration = 0.45);
    void step_to(double target_x, double duration = 0.4);

    bool realtime_;
    std::mt19937 rng_;
    std::vector<Step> script_;
    std::string pending_message_;  // wird dem nächsten gepushten Frame angeheftet
    std::string end_message_;
    std::size_t index_ = 0;
    double x_ = 0.0;
    double t_ = 0.0;
};
