use std::fmt;

use crate::body_state::BodyState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Jump,
    CrouchStart,
    CrouchEnd,
    LaneLeft,
    LaneRight,
}

impl Gesture {
    pub fn as_str(self) -> &'static str {
        match self {
            Gesture::Jump => "jump",
            Gesture::CrouchStart => "crouch_start",
            Gesture::CrouchEnd => "crouch_end",
            Gesture::LaneLeft => "lane_left",
            Gesture::LaneRight => "lane_right",
        }
    }
}

impl fmt::Display for Gesture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub calib_frames: usize,
    pub jump_thresh: f64,
    pub crouch_thresh: f64,
    pub lane_enter: f64,
    pub lane_exit: f64,
    pub jump_cooldown: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calib_frames: 30,
            jump_thresh: 0.10,
            crouch_thresh: 0.25,
            lane_enter: 0.25,
            lane_exit: 0.15,
            jump_cooldown: 0.5,
        }
    }
}

#[derive(Debug)]
pub struct GestureDetector {
    cfg: Config,
    calib_count: usize,
    calib_sum_x: f64,
    calib_sum_h: f64,
    calibrated: bool,
    baseline_x: f64,
    baseline_height: f64,
    lane: i32,
    airborne: bool,
    crouching: bool,
    last_jump_t: f64,
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl GestureDetector {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            calib_count: 0,
            calib_sum_x: 0.0,
            calib_sum_h: 0.0,
            calibrated: false,
            baseline_x: 0.0,
            baseline_height: 0.0,
            lane: 0,
            airborne: false,
            crouching: false,
            last_jump_t: -1e9,
        }
    }

    pub fn update(&mut self, s: BodyState) -> Vec<Gesture> {
        if !self.calibrated {
            self.calib_sum_x += s.x;
            self.calib_sum_h += s.height;
            self.calib_count += 1;

            if self.calib_count >= self.cfg.calib_frames {
                self.baseline_x = self.calib_sum_x / self.calib_count as f64;
                self.baseline_height = self.calib_sum_h / self.calib_count as f64;
                self.calibrated = true;
            }
            return Vec::new();
        }

        let mut events = Vec::new();
        let rel_x = s.x - self.baseline_x;
        let rel_h = s.height - self.baseline_height;

        if !self.airborne && rel_h > self.cfg.jump_thresh {
            if s.t - self.last_jump_t >= self.cfg.jump_cooldown {
                events.push(Gesture::Jump);
                self.last_jump_t = s.t;
            }
            self.airborne = true;
        } else if self.airborne && rel_h < self.cfg.jump_thresh * 0.5 {
            self.airborne = false;
        }

        if !self.crouching && rel_h < -self.cfg.crouch_thresh {
            self.crouching = true;
            events.push(Gesture::CrouchStart);
        } else if self.crouching && rel_h > -(self.cfg.crouch_thresh * 0.7) {
            self.crouching = false;
            events.push(Gesture::CrouchEnd);
        }

        let mut target_lane = self.lane;
        if self.lane == 0 {
            if rel_x < -self.cfg.lane_enter {
                target_lane = -1;
            } else if rel_x > self.cfg.lane_enter {
                target_lane = 1;
            }
        } else if self.lane == -1 && rel_x > -self.cfg.lane_exit {
            target_lane = 0;
        } else if self.lane == 1 && rel_x < self.cfg.lane_exit {
            target_lane = 0;
        }

        if target_lane < self.lane {
            events.push(Gesture::LaneLeft);
        } else if target_lane > self.lane {
            events.push(Gesture::LaneRight);
        }
        self.lane = target_lane;

        events
    }

    pub fn calibrated(&self) -> bool {
        self.calibrated
    }

    pub fn baseline_x(&self) -> f64 {
        self.baseline_x
    }

    pub fn baseline_height(&self) -> f64 {
        self.baseline_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FPS: f64 = 30.0;
    const DT: f64 = 1.0 / FPS;
    const STAND: f64 = 1.75;

    struct Feeder {
        det: GestureDetector,
        t: f64,
    }

    impl Feeder {
        fn new() -> Self {
            Self {
                det: GestureDetector::new(Config {
                    calib_frames: 10,
                    ..Config::default()
                }),
                t: 0.0,
            }
        }

        fn feed(&mut self, x: f64, height: f64, frames: usize) -> Vec<Gesture> {
            let mut events = Vec::new();
            for _ in 0..frames {
                events.extend(self.det.update(BodyState::new(x, height, self.t)));
                self.t += DT;
            }
            events
        }

        fn calibrate(&mut self) {
            self.feed(0.0, STAND, 10);
            assert!(self.det.calibrated());
        }
    }

    fn count(events: &[Gesture], needle: Gesture) -> usize {
        events.iter().filter(|&&event| event == needle).count()
    }

    #[test]
    fn keine_events_waehrend_kalibrierung() {
        let mut f = Feeder::new();
        let events = f.feed(0.0, STAND, 9);
        assert_eq!(events.len(), 0);
        assert!(!f.det.calibrated());
    }

    #[test]
    fn sprung_feuert_genau_einmal() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut events = f.feed(0.0, STAND + 0.25, 8);
        events.extend(f.feed(0.0, STAND, 5));
        assert_eq!(count(&events, Gesture::Jump), 1);
    }

    #[test]
    fn sprung_cooldown_blockt_doppelausloesung() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut events = f.feed(0.0, STAND + 0.25, 3);
        events.extend(f.feed(0.0, STAND, 2));
        events.extend(f.feed(0.0, STAND + 0.25, 3));
        assert_eq!(count(&events, Gesture::Jump), 1);
    }

    #[test]
    fn zwei_spruenge_nach_cooldown() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut events = f.feed(0.0, STAND + 0.25, 5);
        events.extend(f.feed(0.0, STAND, 20));
        events.extend(f.feed(0.0, STAND + 0.25, 5));
        assert_eq!(count(&events, Gesture::Jump), 2);
    }

    #[test]
    fn ducken_start_und_ende() {
        let mut f = Feeder::new();
        f.calibrate();
        let events = f.feed(0.0, STAND - 0.40, 10);
        assert_eq!(count(&events, Gesture::CrouchStart), 1);
        assert_eq!(count(&events, Gesture::CrouchEnd), 0);

        let events = f.feed(0.0, STAND, 5);
        assert_eq!(count(&events, Gesture::CrouchEnd), 1);
    }

    #[test]
    fn spurwechsel_links_und_zurueck() {
        let mut f = Feeder::new();
        f.calibrate();
        let events = f.feed(-0.50, STAND, 5);
        assert_eq!(count(&events, Gesture::LaneLeft), 1);

        let events = f.feed(0.0, STAND, 5);
        assert_eq!(count(&events, Gesture::LaneRight), 1);
    }

    #[test]
    fn hysterese_kein_flackern_an_der_spurgrenze() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut events = f.feed(-0.30, STAND, 3);
        events.extend(f.feed(-0.20, STAND, 3));
        events.extend(f.feed(-0.30, STAND, 3));
        assert_eq!(count(&events, Gesture::LaneLeft), 1);
        assert_eq!(count(&events, Gesture::LaneRight), 0);
    }

    #[test]
    fn kleine_bewegungen_loesen_nichts_aus() {
        let mut f = Feeder::new();
        f.calibrate();
        let events = f.feed(0.05, STAND + 0.03, 30);
        assert_eq!(events.len(), 0);
    }
}
