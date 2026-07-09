//! Rule-based gesture detection on ZONE RATIOS (port of detector.py).
//!
//! Operates purely on zone ratios (0..1) and is independent of the source.
//! Emits abstract events; the mapping event -> key happens in the pipeline.

use std::fmt;

use crate::zones::ZoneActivity;

/// Four zones with fixed indices: 0=left 1=right 2=up 3=down.
const LEFT: usize = 0;
const RIGHT: usize = 1;
const UP: usize = 2;
const DOWN: usize = 3;
const TAP_ZONES: [usize; 3] = [LEFT, RIGHT, UP];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    TapLeft,
    TapRight,
    TapUp,
    HoldDownStart,
    HoldDownEnd,
}

impl Event {
    pub fn as_str(self) -> &'static str {
        match self {
            Event::TapLeft => "tap_left",
            Event::TapRight => "tap_right",
            Event::TapUp => "tap_up",
            Event::HoldDownStart => "hold_down_start",
            Event::HoldDownEnd => "hold_down_end",
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub calib_frames: usize,
    /// Enter threshold per zone [left, right, up, down].
    pub enter_ratio: [f64; 4],
    /// Exit threshold per zone [left, right, up, down].
    pub exit_ratio: [f64; 4],
    pub cooldown_s: f64,
    /// An edge counts only after N consecutive frames above/below the
    /// threshold (debounce against single-frame outliers; 1 = immediate).
    pub confirm_frames: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calib_frames: 30, // ~1s @30FPS
            enter_ratio: [0.15; 4],
            exit_ratio: [0.08; 4],
            cooldown_s: 0.5,
            confirm_frames: 2,
        }
    }
}

#[derive(Debug)]
pub struct GestureDetector {
    cfg: Config,
    calib_count: usize,
    calib_sum: [f64; 4],
    calibrated: bool,
    baseline: [f64; 4],
    active: [bool; 4],
    last_tap_t: [f64; 4],
    /// Consecutive frames above enter (zone inactive) resp. below exit (zone active).
    above_count: [u32; 4],
    below_count: [u32; 4],
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
            calib_sum: [0.0; 4],
            calibrated: false,
            baseline: [0.0; 4],
            active: [false; 4],
            last_tap_t: [-1e9; 4],
            above_count: [0; 4],
            below_count: [0; 4],
        }
    }

    pub fn calibrated(&self) -> bool {
        self.calibrated
    }

    pub fn baseline(&self) -> [f64; 4] {
        self.baseline
    }

    /// Relearn the idle baseline (e.g. via the 'c' key).
    pub fn start_recalibration(&mut self) {
        self.calibrated = false;
        self.calib_count = 0;
        self.calib_sum = [0.0; 4];
        self.above_count = [0; 4];
        self.below_count = [0; 4];
    }

    pub fn update(&mut self, s: ZoneActivity) -> Vec<Event> {
        let raw = [s.zones.left, s.zones.right, s.zones.up, s.zones.down];

        if !self.calibrated {
            for i in 0..4 {
                self.calib_sum[i] += raw[i];
            }
            self.calib_count += 1;
            if self.calib_count >= self.cfg.calib_frames {
                for i in 0..4 {
                    self.baseline[i] = self.calib_sum[i] / self.calib_count as f64;
                }
                self.calibrated = true;
            }
            return Vec::new();
        }

        let mut eff = [0.0; 4];
        for i in 0..4 {
            eff[i] = (raw[i] - self.baseline[i]).max(0.0);
        }

        // Hysteresis state per zone + edges. An edge counts only after
        // confirm_frames consecutive frames (debounce); tap zones additionally
        // need the source's direction gate (dir_ok). The gate does not discard
        // the confirmation - if the direction matches shortly after, the edge
        // fires late; if the motion ends, eff drops below the threshold anyway
        // and the counter resets.
        let need = self.cfg.confirm_frames.max(1);
        let mut rising = [false; 4];
        let mut falling = [false; 4];
        for i in 0..4 {
            if !self.active[i] {
                self.below_count[i] = 0;
                if eff[i] > self.cfg.enter_ratio[i] {
                    self.above_count[i] = (self.above_count[i] + 1).min(need);
                } else {
                    self.above_count[i] = 0;
                }
                let dir_ok = i == DOWN || s.dir_ok[i];
                if self.above_count[i] >= need && dir_ok {
                    self.active[i] = true;
                    rising[i] = true;
                    self.above_count[i] = 0;
                }
            } else {
                self.above_count[i] = 0;
                if eff[i] < self.cfg.exit_ratio[i] {
                    self.below_count[i] += 1;
                } else {
                    self.below_count[i] = 0;
                }
                if self.below_count[i] >= need {
                    self.active[i] = false;
                    falling[i] = true;
                    self.below_count[i] = 0;
                }
            }
        }

        let mut events = Vec::new();

        // Tap zones: ONE tap per rising edge; on conflict the strongest zone wins.
        let mut winner: Option<usize> = None;
        for &z in &TAP_ZONES {
            if rising[z] && (winner.is_none() || eff[z] > eff[winner.unwrap()]) {
                winner = Some(z);
            }
        }
        if let Some(z) = winner {
            if s.t - self.last_tap_t[z] >= self.cfg.cooldown_s {
                events.push(match z {
                    LEFT => Event::TapLeft,
                    RIGHT => Event::TapRight,
                    _ => Event::TapUp,
                });
                self.last_tap_t[z] = s.t;
            }
        }

        if rising[DOWN] {
            events.push(Event::HoldDownStart);
        } else if falling[DOWN] {
            events.push(Event::HoldDownEnd);
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::{ZoneActivity, Zones};

    const FPS: f64 = 30.0;
    const DT: f64 = 1.0 / FPS;

    struct Feeder {
        det: GestureDetector,
        t: f64,
    }

    impl Feeder {
        fn new() -> Self {
            Self::with_cfg(Config {
                calib_frames: 10,
                ..Config::default()
            })
        }

        fn with_cfg(cfg: Config) -> Self {
            Self {
                det: GestureDetector::new(cfg),
                t: 0.0,
            }
        }

        fn feed(&mut self, z: Zones, frames: usize) -> Vec<Event> {
            self.feed_gated(z, [true; 3], frames)
        }

        fn feed_gated(&mut self, z: Zones, dir_ok: [bool; 3], frames: usize) -> Vec<Event> {
            let mut events = Vec::new();
            for _ in 0..frames {
                events.extend(
                    self.det
                        .update(ZoneActivity::new(z, self.t).with_dir_ok(dir_ok)),
                );
                self.t += DT;
            }
            events
        }

        fn calibrate(&mut self) {
            self.feed(Zones::default(), 10);
            assert!(self.det.calibrated());
        }
    }

    fn count(events: &[Event], needle: Event) -> usize {
        events.iter().filter(|&&e| e == needle).count()
    }

    #[test]
    fn keine_events_waehrend_kalibrierung() {
        let mut f = Feeder::new();
        let ev = f.feed(Zones::default(), 9);
        assert_eq!(ev.len(), 0);
        assert!(!f.det.calibrated());
    }

    #[test]
    fn right_feuert_genau_einmal() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 8);
        assert_eq!(count(&ev, Event::TapRight), 1);
    }

    #[test]
    fn cooldown_blockt_doppelausloesung() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut ev = f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 3);
        ev.extend(f.feed(Zones::default(), 2));
        ev.extend(f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 3));
        assert_eq!(count(&ev, Event::TapRight), 1);
    }

    #[test]
    fn zwei_taps_nach_cooldown() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut ev = f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 3);
        ev.extend(f.feed(Zones::default(), 18));
        ev.extend(f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 3));
        assert_eq!(count(&ev, Event::TapRight), 2);
    }

    #[test]
    fn hold_down_start_und_end() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed(Zones::new(0.0, 0.0, 0.0, 0.9), 10);
        assert_eq!(count(&ev, Event::HoldDownStart), 1);
        assert_eq!(count(&ev, Event::HoldDownEnd), 0);
        let ev = f.feed(Zones::default(), 5);
        assert_eq!(count(&ev, Event::HoldDownEnd), 1);
    }

    #[test]
    fn hysterese_kein_flackern() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut ev = f.feed(Zones::new(0.16, 0.0, 0.0, 0.0), 3);
        ev.extend(f.feed(Zones::new(0.10, 0.0, 0.0, 0.0), 3));
        ev.extend(f.feed(Zones::new(0.16, 0.0, 0.0, 0.0), 3));
        assert_eq!(count(&ev, Event::TapLeft), 1);
    }

    #[test]
    fn konflikt_staerkere_zone_gewinnt() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed(Zones::new(0.7, 0.3, 0.0, 0.0), 3);
        assert_eq!(count(&ev, Event::TapLeft), 1);
        assert_eq!(count(&ev, Event::TapRight), 0);
    }

    #[test]
    fn kleine_bewegungen_loesen_nichts_aus() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed(Zones::new(0.05, 0.04, 0.03, 0.02), 30);
        assert_eq!(ev.len(), 0);
    }

    #[test]
    fn up_feuert_tap() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed(Zones::new(0.0, 0.0, 0.6, 0.0), 6);
        assert_eq!(count(&ev, Event::TapUp), 1);
    }

    // 2-frame confirmation filters single-frame outliers.
    #[test]
    fn einzelner_ausreisser_frame_loest_nichts_aus() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut ev = f.feed(Zones::new(0.0, 0.7, 0.0, 0.0), 1);
        ev.extend(f.feed(Zones::default(), 10));
        assert_eq!(ev.len(), 0);
    }

    // A single-frame mask dropout must not end a hold.
    #[test]
    fn hold_uebersteht_einzel_frame_dropout() {
        let mut f = Feeder::new();
        f.calibrate();
        let mut ev = f.feed(Zones::new(0.0, 0.0, 0.0, 0.9), 5);
        ev.extend(f.feed(Zones::default(), 1)); // mask drops out for one frame
        ev.extend(f.feed(Zones::new(0.0, 0.0, 0.0, 0.9), 5));
        assert_eq!(count(&ev, Event::HoldDownStart), 1);
        assert_eq!(count(&ev, Event::HoldDownEnd), 0);
    }

    // The direction gate blocks taps whose motion does not match.
    #[test]
    fn flow_gate_blockt_tap_gegen_richtung() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed_gated(Zones::new(0.0, 0.7, 0.0, 0.0), [true, false, true], 6);
        assert_eq!(count(&ev, Event::TapRight), 0);
        // Direction matches now -> the confirmed edge fires late.
        let ev = f.feed_gated(Zones::new(0.0, 0.7, 0.0, 0.0), [true, true, true], 2);
        assert_eq!(count(&ev, Event::TapRight), 1);
    }

    // The gate applies only to tap zones, never to the hold (down).
    #[test]
    fn flow_gate_wirkt_nicht_auf_hold() {
        let mut f = Feeder::new();
        f.calibrate();
        let ev = f.feed_gated(Zones::new(0.0, 0.0, 0.0, 0.9), [false, false, false], 6);
        assert_eq!(count(&ev, Event::HoldDownStart), 1);
    }

    // Per-zone thresholds apply independently of each other.
    #[test]
    fn pro_zone_schwellen_wirken() {
        let mut f = Feeder::with_cfg(Config {
            calib_frames: 10,
            enter_ratio: [0.15, 0.15, 0.10, 0.22],
            exit_ratio: [0.08, 0.08, 0.05, 0.08],
            ..Config::default()
        });
        f.calibrate();
        // up: 0.12 is above the up threshold 0.10 -> tap
        let ev = f.feed(Zones::new(0.0, 0.0, 0.12, 0.0), 6);
        assert_eq!(count(&ev, Event::TapUp), 1);
        // down: 0.18 stays below the down threshold 0.22 -> no hold
        let ev = f.feed(Zones::new(0.0, 0.0, 0.0, 0.18), 10);
        assert_eq!(count(&ev, Event::HoldDownStart), 0);
    }
}
