#include "sources/mock_source.h"

#include <chrono>
#include <cmath>
#include <cstdio>
#include <numbers>
#include <thread>
#include <utility>

MockSource::MockSource(bool realtime)
    : realtime_(realtime), rng_(std::random_device{}()) {
    say(">> Simulation: Person steht still (Kalibrierung)...");
    hold(1.5);
    say(">> Simulation: SPRUNG");
    jump();
    hold(1.0);
    say(">> Simulation: Schritt nach LINKS");
    step_to(-kSideX);
    hold(0.8);
    say(">> Simulation: zurück zur Mitte");
    step_to(0.0);
    hold(0.8);
    say(">> Simulation: DUCKEN (1.2s)");
    hold(1.2, kCrouchHeight);
    say(">> Simulation: wieder aufstehen");
    hold(1.0);
    say(">> Simulation: Schritt nach RECHTS");
    step_to(+kSideX);
    hold(0.8);
    say(">> Simulation: zurück zur Mitte");
    step_to(0.0);
    hold(0.5);
    say(">> Simulation: noch ein SPRUNG");
    jump();
    hold(1.0);
    end_message_ = ">> Simulation beendet.";
}

double MockSource::noise() {
    // Sensor-Rauschen wie im Python-Prototyp: gleichverteilt in [-0.01, +0.01]
    return std::uniform_real_distribution<double>(-0.01, 0.01)(rng_);
}

void MockSource::say(std::string message) {
    pending_message_ = std::move(message);
}

void MockSource::push(double x, double height) {
    script_.push_back(Step{std::move(pending_message_), x + noise(), height + noise()});
    pending_message_.clear();
}

void MockSource::hold(double seconds, double height) {
    const int n = static_cast<int>(seconds * kFps);
    for (int i = 0; i < n; ++i) {
        push(x_, height);
    }
}

void MockSource::jump(double duration) {
    const int n = static_cast<int>(duration * kFps);
    for (int i = 0; i < n; ++i) {
        const double p = static_cast<double>(i) / (n - 1);
        // Parabel rauf und runter
        push(x_, kStandHeight + kJumpPeak * std::sin(std::numbers::pi * p));
    }
}

void MockSource::step_to(double target_x, double duration) {
    const int n = static_cast<int>(duration * kFps);
    const double start = x_;
    for (int i = 0; i < n; ++i) {
        x_ = start + (target_x - start) * (i + 1) / n;
        push(x_, kStandHeight);
    }
}

bool MockSource::next(BodyState& out) {
    if (index_ >= script_.size()) {
        if (!end_message_.empty()) {
            std::printf("%s\n", end_message_.c_str());
            end_message_.clear();
        }
        return false;
    }

    const Step& step = script_[index_++];
    if (!step.message.empty()) {
        std::printf("%s\n", step.message.c_str());
    }
    out = BodyState{step.x, step.height, t_};
    t_ += kDt;

    if (realtime_) {
        std::this_thread::sleep_for(std::chrono::duration<double>(kDt));
    }
    return true;
}
