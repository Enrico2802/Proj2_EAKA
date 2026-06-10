// Kinect-Steuerungs-Prototyp (C++-Port): Gesten -> Tastatur.
//
// Pipeline (identisch zur Zielarchitektur aus plan.md):
//
//     Quelle (Mock / später Kinect)  ->  GestureDetector  ->  KeySender (SendInput)
//
// Beispiele:
//     kinect-input                  # Mock-Drehbuch, nur Logging (Dry-Run)
//     kinect-input --send           # Mock-Drehbuch, sendet ECHTE Tasten (3s Zeit, Zielfenster zu fokussieren)
//     kinect-input --source manual  # Person interaktiv per w/a/s/d steuern (Dry-Run)

#include <chrono>
#include <csignal>
#include <cstdio>
#include <memory>
#include <string>
#include <thread>
#include <vector>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "gesture_detector.h"
#include "key_sender.h"
#include "sources/manual_source.h"
#include "sources/mock_source.h"
#include "sources/source.h"

namespace {

// Wird per Ctrl+C gesetzt; die Hauptschleife prüft das Flag und beendet sich
// sauber (Pendant zum KeyboardInterrupt-Handling in Python).
volatile std::sig_atomic_t g_stop = 0;

void on_sigint(int) { g_stop = 1; }

// Sicherheitsnetz wie das try/finally in Python: Ctrl nie gedrückt zurücklassen.
//
// C++-Besonderheit (RAII): Der Aufräumcode steckt im Destruktor und läuft
// automatisch, sobald der Scope verlassen wird - egal ob regulär, per break
// oder durch eine Exception.
struct CtrlReleaseGuard {
    KeySender& sender;
    ~CtrlReleaseGuard() { sender.release(Key::Ctrl); }
};

// Mapping Geste -> Taste (muss zur Tastenbelegung des Spiels passen).
struct Action {
    enum class Kind { Tap, Press, Release } kind;
    Key key;
};

const char* to_string(Action::Kind k) {
    switch (k) {
        case Action::Kind::Tap:     return "tap";
        case Action::Kind::Press:   return "press";
        case Action::Kind::Release: return "release";
    }
    return "?";
}

Action action_for(Gesture g) {
    switch (g) {
        case Gesture::Jump:        return {Action::Kind::Tap, Key::Space};
        case Gesture::LaneLeft:    return {Action::Kind::Tap, Key::A};
        case Gesture::LaneRight:   return {Action::Kind::Tap, Key::D};
        case Gesture::CrouchStart: return {Action::Kind::Press, Key::Ctrl};
        case Gesture::CrouchEnd:   return {Action::Kind::Release, Key::Ctrl};
    }
    return {Action::Kind::Tap, Key::Space};  // unerreichbar, beruhigt den Compiler
}

int usage(const char* prog) {
    std::printf("Verwendung: %s [--source mock|manual] [--send]\n", prog);
    std::printf("  --source  Datenquelle: mock = Drehbuch-Simulation, manual = per w/a/s/d steuern\n");
    std::printf("  --send    Tasten WIRKLICH senden (Standard: Dry-Run, nur Logging)\n");
    return 2;
}

}  // namespace

int main(int argc, char** argv) {
    SetConsoleOutputCP(CP_UTF8);  // deutsche Umlaute im Konsolenfenster

    std::string source_name = "mock";
    bool send = false;
    for (int i = 1; i < argc; ++i) {
        const std::string arg = argv[i];
        if (arg == "--send") {
            send = true;
        } else if (arg == "--source" && i + 1 < argc) {
            source_name = argv[++i];
        } else {
            return usage(argv[0]);
        }
    }
    if (source_name != "mock" && source_name != "manual") {
        return usage(argv[0]);
    }

    KeySender sender(/*dry_run=*/!send);
    GestureDetector detector;

    // C++-Hinweis: unique_ptr<Source> besitzt die Quelle und gibt sie am Ende
    // automatisch frei (RAII statt Garbage Collector).
    std::unique_ptr<Source> source;
    if (source_name == "mock") {
        source = std::make_unique<MockSource>();
    } else {
        source = std::make_unique<ManualSource>();
    }

    if (send) {
        std::printf("ECHTER Tastatur-Modus! Fokussiere jetzt das Zielfenster (z.B. Notepad oder das Spiel)...\n");
        for (int i = 3; i >= 1; --i) {
            std::printf("  Start in %d...\n", i);
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    }

    std::printf("Kalibrierung läuft - bitte still stehen...\n");
    bool was_calibrated = false;

    std::signal(SIGINT, on_sigint);
    {
        CtrlReleaseGuard guard{sender};

        BodyState state;
        while (!g_stop && source->next(state)) {
            const std::vector<Gesture> events = detector.update(state);

            if (detector.calibrated() && !was_calibrated) {
                was_calibrated = true;
                std::printf("Kalibriert: Baseline-Höhe = %.2f, Baseline-x = %+.2f\n\n",
                            detector.baseline_height(), detector.baseline_x());
            }

            for (const Gesture g : events) {
                const Action a = action_for(g);
                std::printf("[%6.2fs] GESTE: %-13s -> %s '%s'\n",
                            state.t, to_string(g), to_string(a.kind), to_string(a.key));
                switch (a.kind) {
                    case Action::Kind::Tap:     sender.tap(a.key); break;
                    case Action::Kind::Press:   sender.press(a.key); break;
                    case Action::Kind::Release: sender.release(a.key); break;
                }
            }
        }
    }  // <- hier läuft der Destruktor des Guards und gibt Ctrl frei

    std::printf("\nFertig.\n");
    return 0;
}
