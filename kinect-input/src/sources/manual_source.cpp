#include "sources/manual_source.h"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cwctype>
#include <numbers>
#include <thread>

#include <conio.h>  // _kbhit/_getwch - das C++-Pendant zu Pythons msvcrt

bool ManualSource::next(BodyState& out) {
    if (first_call_) {
        first_call_ = false;
        std::printf("Steuerung: [w] springen  [s] ducken an/aus  [a]/[d] Schritt links/rechts  [q] beenden\n");
    }

    // Alle seit dem letzten Frame gedrückten Konsolen-Tasten abarbeiten.
    while (_kbhit()) {
        const wint_t key = std::towlower(_getwch());
        if (key == L'q') {
            return false;
        } else if (key == L'w' && jump_t_ < 0) {
            jump_t_ = t_;
        } else if (key == L's') {
            crouching_ = !crouching_;
        } else if (key == L'a') {
            x_ = std::max(-kSideX, x_ - kSideX);
        } else if (key == L'd') {
            x_ = std::min(+kSideX, x_ + kSideX);
        }
    }

    double height = crouching_ ? kCrouchHeight : kStandHeight;
    if (jump_t_ >= 0) {
        const double p = (t_ - jump_t_) / 0.45;  // Sprungdauer 0.45s
        if (p >= 1.0) {
            jump_t_ = -1.0;
        } else {
            height = kStandHeight + kJumpPeak * std::sin(std::numbers::pi * p);
        }
    }

    out = BodyState{x_, height, t_};
    t_ += kDt;
    std::this_thread::sleep_for(std::chrono::duration<double>(kDt));
    return true;
}
