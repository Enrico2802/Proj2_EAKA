#include "gesture_detector.h"

const char* to_string(Gesture g) {
    switch (g) {
        case Gesture::Jump:        return "jump";
        case Gesture::CrouchStart: return "crouch_start";
        case Gesture::CrouchEnd:   return "crouch_end";
        case Gesture::LaneLeft:    return "lane_left";
        case Gesture::LaneRight:   return "lane_right";
    }
    return "?";
}

GestureDetector::GestureDetector() : GestureDetector(Config{}) {}

GestureDetector::GestureDetector(Config cfg) : cfg_(cfg) {}

std::vector<Gesture> GestureDetector::update(const BodyState& s) {
    if (!calibrated_) {
        calib_sum_x_ += s.x;
        calib_sum_h_ += s.height;
        ++calib_count_;
        if (calib_count_ >= cfg_.calib_frames) {
            baseline_x_ = calib_sum_x_ / calib_count_;
            baseline_height_ = calib_sum_h_ / calib_count_;
            calibrated_ = true;
        }
        return {};
    }

    std::vector<Gesture> events;
    const double rel_x = s.x - baseline_x_;
    const double rel_h = s.height - baseline_height_;

    // --- Springen: steigende Flanke der Körperhöhe + Cooldown ---
    if (!airborne_ && rel_h > cfg_.jump_thresh) {
        if (s.t - last_jump_t_ >= cfg_.jump_cooldown) {
            events.push_back(Gesture::Jump);
            last_jump_t_ = s.t;
        }
        airborne_ = true;
    } else if (airborne_ && rel_h < cfg_.jump_thresh * 0.5) {
        airborne_ = false;  // gelandet
    }

    // --- Ducken: zustandsbasiert mit Hysterese (Taste wird gehalten) ---
    if (!crouching_ && rel_h < -cfg_.crouch_thresh) {
        crouching_ = true;
        events.push_back(Gesture::CrouchStart);
    } else if (crouching_ && rel_h > -(cfg_.crouch_thresh * 0.7)) {
        crouching_ = false;
        events.push_back(Gesture::CrouchEnd);
    }

    // --- Spurwechsel: physische Position -> Spur, mit Hysterese ---
    int target_lane = lane_;
    if (lane_ == 0) {
        if (rel_x < -cfg_.lane_enter) {
            target_lane = -1;
        } else if (rel_x > cfg_.lane_enter) {
            target_lane = +1;
        }
    } else if (lane_ == -1 && rel_x > -cfg_.lane_exit) {
        target_lane = 0;
    } else if (lane_ == +1 && rel_x < cfg_.lane_exit) {
        target_lane = 0;
    }

    if (target_lane < lane_) {
        events.push_back(Gesture::LaneLeft);
    } else if (target_lane > lane_) {
        events.push_back(Gesture::LaneRight);
    }
    lane_ = target_lane;

    return events;
}
