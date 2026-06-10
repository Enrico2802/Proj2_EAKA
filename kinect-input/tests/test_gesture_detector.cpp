// Unit-Tests für die Gestenerkennung - Port von prototyp/test_gesture_detector.py.
//
// Bewusst ohne Test-Framework: einfache CHECK-Makros, Exit-Code != 0 bei
// Fehlschlag (genau das wertet auch CTest aus). Jeder Test bekommt einen
// frischen Feeder (Pendant zu unittest.setUp).

#include <cstdio>
#include <iterator>
#include <vector>

#include "body_state.h"
#include "gesture_detector.h"

namespace {

constexpr int    kFps = 30;
constexpr double kDt = 1.0 / kFps;
constexpr double kStand = 1.75;

int g_checks_failed = 0;

#define CHECK(cond)                                                              \
    do {                                                                         \
        if (!(cond)) {                                                           \
            std::printf("    FEHLER %s:%d: %s\n", __FILE__, __LINE__, #cond);    \
            ++g_checks_failed;                                                   \
        }                                                                        \
    } while (0)

#define CHECK_EQ(actual, expected)                                               \
    do {                                                                         \
        const long long a_ = static_cast<long long>(actual);                     \
        const long long e_ = static_cast<long long>(expected);                   \
        if (a_ != e_) {                                                          \
            std::printf("    FEHLER %s:%d: %s (ist %lld, erwartet %lld)\n",      \
                        __FILE__, __LINE__, #actual, a_, e_);                    \
            ++g_checks_failed;                                                   \
        }                                                                        \
    } while (0)

// Schickt Frames durch den Detektor und sammelt alle Events (wie feed() im Python-Test).
struct Feeder {
    GestureDetector det{GestureDetector::Config{.calib_frames = 10}};
    double t = 0.0;

    std::vector<Gesture> feed(double x, double height, int frames = 1) {
        std::vector<Gesture> events;
        for (int i = 0; i < frames; ++i) {
            const std::vector<Gesture> e = det.update(BodyState{x, height, t});
            events.insert(events.end(), e.begin(), e.end());
            t += kDt;
        }
        return events;
    }

    void calibrate() {
        feed(0.0, kStand, 10);
        CHECK(det.calibrated());
    }
};

int count(const std::vector<Gesture>& events, Gesture g) {
    int n = 0;
    for (const Gesture e : events) {
        if (e == g) ++n;
    }
    return n;
}

void append(std::vector<Gesture>& into, const std::vector<Gesture>& more) {
    into.insert(into.end(), more.begin(), more.end());
}

void test_keine_events_waehrend_kalibrierung() {
    Feeder f;
    const std::vector<Gesture> events = f.feed(0.0, kStand, 9);
    CHECK_EQ(events.size(), 0);
    CHECK(!f.det.calibrated());
}

void test_sprung_feuert_genau_einmal() {
    Feeder f;
    f.calibrate();
    std::vector<Gesture> events = f.feed(0.0, kStand + 0.25, 8);  // in der Luft
    append(events, f.feed(0.0, kStand, 5));                       // gelandet
    CHECK_EQ(count(events, Gesture::Jump), 1);
}

void test_sprung_cooldown_blockt_doppelausloesung() {
    Feeder f;
    f.calibrate();
    std::vector<Gesture> events = f.feed(0.0, kStand + 0.25, 3);
    append(events, f.feed(0.0, kStand, 2));         // landet sofort wieder
    append(events, f.feed(0.0, kStand + 0.25, 3));  // zweiter "Sprung" innerhalb des Cooldowns
    CHECK_EQ(count(events, Gesture::Jump), 1);
}

void test_zwei_spruenge_nach_cooldown() {
    Feeder f;
    f.calibrate();
    std::vector<Gesture> events = f.feed(0.0, kStand + 0.25, 5);
    append(events, f.feed(0.0, kStand, 20));        // > 0.5s Cooldown abwarten
    append(events, f.feed(0.0, kStand + 0.25, 5));
    CHECK_EQ(count(events, Gesture::Jump), 2);
}

void test_ducken_start_und_ende() {
    Feeder f;
    f.calibrate();
    std::vector<Gesture> events = f.feed(0.0, kStand - 0.40, 10);
    CHECK_EQ(count(events, Gesture::CrouchStart), 1);
    CHECK_EQ(count(events, Gesture::CrouchEnd), 0);
    events = f.feed(0.0, kStand, 5);
    CHECK_EQ(count(events, Gesture::CrouchEnd), 1);
}

void test_spurwechsel_links_und_zurueck() {
    Feeder f;
    f.calibrate();
    std::vector<Gesture> events = f.feed(-0.50, kStand, 5);
    CHECK_EQ(count(events, Gesture::LaneLeft), 1);
    events = f.feed(0.0, kStand, 5);
    CHECK_EQ(count(events, Gesture::LaneRight), 1);
}

void test_hysterese_kein_flackern_an_der_spurgrenze() {
    Feeder f;
    f.calibrate();
    // Knapp über der Eintritts-Schwelle, dann knapp darunter (aber über der Austritts-Schwelle):
    std::vector<Gesture> events = f.feed(-0.30, kStand, 3);
    append(events, f.feed(-0.20, kStand, 3));
    append(events, f.feed(-0.30, kStand, 3));
    CHECK_EQ(count(events, Gesture::LaneLeft), 1);
    CHECK_EQ(count(events, Gesture::LaneRight), 0);
}

void test_kleine_bewegungen_loesen_nichts_aus() {
    Feeder f;
    f.calibrate();
    const std::vector<Gesture> events = f.feed(0.05, kStand + 0.03, 30);
    CHECK_EQ(events.size(), 0);
}

struct TestCase {
    const char* name;
    void (*fn)();
};

}  // namespace

int main() {
    const TestCase tests[] = {
        {"keine_events_waehrend_kalibrierung", test_keine_events_waehrend_kalibrierung},
        {"sprung_feuert_genau_einmal", test_sprung_feuert_genau_einmal},
        {"sprung_cooldown_blockt_doppelausloesung", test_sprung_cooldown_blockt_doppelausloesung},
        {"zwei_spruenge_nach_cooldown", test_zwei_spruenge_nach_cooldown},
        {"ducken_start_und_ende", test_ducken_start_und_ende},
        {"spurwechsel_links_und_zurueck", test_spurwechsel_links_und_zurueck},
        {"hysterese_kein_flackern_an_der_spurgrenze", test_hysterese_kein_flackern_an_der_spurgrenze},
        {"kleine_bewegungen_loesen_nichts_aus", test_kleine_bewegungen_loesen_nichts_aus},
    };

    int tests_failed = 0;
    for (const TestCase& t : tests) {
        const int before = g_checks_failed;
        t.fn();
        const bool ok = (g_checks_failed == before);
        std::printf("[%s] %s\n", ok ? "OK" : "FEHLGESCHLAGEN", t.name);
        if (!ok) ++tests_failed;
    }

    const int total = static_cast<int>(std::size(tests));
    std::printf("\n%d/%d Tests bestanden\n", total - tests_failed, total);
    return tests_failed == 0 ? 0 : 1;
}
