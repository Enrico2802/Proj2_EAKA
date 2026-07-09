//! Camera-free sources: scripted mock and manual keyboard control
//! (port of sources.py). Both deliver ZoneActivity like the webcam source.

use std::thread;
use std::time::Duration;

use crate::zones::{ZoneActivity, Zones};

const FPS: f64 = 30.0;
const DT: f64 = 1.0 / FPS;

struct Segment {
    label: &'static str,
    zones: Zones,
    frames: usize,
}

/// Plays back a fixed script of zone activities.
pub struct MockSource {
    segs: Vec<Segment>,
    seg_i: usize,
    frame_i: usize,
    t: f64,
    realtime: bool,
}

impl MockSource {
    pub fn new(realtime: bool) -> Self {
        let hold = |secs: f64| (secs * FPS) as usize;
        let segs = vec![
            Segment { label: "Ruhe (Kalibrierung)...", zones: Zones::default(), frames: hold(1.5) },
            Segment { label: "Arme HOCH -> Sprung", zones: Zones::new(0.0, 0.0, 0.6, 0.0), frames: hold(0.3) },
            Segment { label: "", zones: Zones::default(), frames: hold(1.0) },
            Segment { label: "Arm LINKS -> A", zones: Zones::new(0.7, 0.0, 0.0, 0.0), frames: hold(0.3) },
            Segment { label: "", zones: Zones::default(), frames: hold(1.0) },
            Segment { label: "Arm RECHTS -> D", zones: Zones::new(0.0, 0.7, 0.0, 0.0), frames: hold(0.3) },
            Segment { label: "", zones: Zones::default(), frames: hold(1.0) },
            Segment { label: "DUCKEN -> HOLD S", zones: Zones::new(0.0, 0.0, 0.0, 0.9), frames: hold(1.2) },
            Segment { label: "aufstehen -> HOLD S aus", zones: Zones::default(), frames: hold(1.0) },
            Segment { label: "LINKS & rechts gleichzeitig -> nur A", zones: Zones::new(0.7, 0.3, 0.0, 0.0), frames: hold(0.3) },
            Segment { label: "", zones: Zones::default(), frames: hold(0.5) },
        ];
        Self { segs, seg_i: 0, frame_i: 0, t: 0.0, realtime }
    }

    pub fn next(&mut self) -> Option<ZoneActivity> {
        if self.seg_i >= self.segs.len() {
            return None;
        }
        if self.frame_i == 0 {
            let label = self.segs[self.seg_i].label;
            if !label.is_empty() {
                println!(">> Mock: {label}");
            }
        }
        let zones = self.segs[self.seg_i].zones;
        let s = ZoneActivity::new(zones, self.t);

        self.frame_i += 1;
        if self.frame_i >= self.segs[self.seg_i].frames {
            self.seg_i += 1;
            self.frame_i = 0;
        }
        self.t += DT;
        if self.realtime {
            thread::sleep(Duration::from_secs_f64(DT));
        }
        Some(s)
    }
}

/// Manual control via console keys (w/a/d = up/left/right, s = toggle crouch,
/// c = recalibrate, q = quit). Non-blocking via the UCRT.
pub struct ManualSource {
    t: f64,
    down: bool,
    pub want_recalib: bool,
}

impl ManualSource {
    pub fn new() -> Self {
        println!("Steuerung: [w]=oben [a]=links [d]=rechts [s]=ducken an/aus [c]=Kalib [q]=Ende");
        Self { t: 0.0, down: false, want_recalib: false }
    }

    pub fn next(&mut self) -> Option<ZoneActivity> {
        let mut z = Zones::default();
        self.want_recalib = false;
        while console::kbhit() {
            match console::getch() {
                b'q' | b'Q' => return None,
                b'w' | b'W' => z.up = 0.9,
                b'a' | b'A' => z.left = 0.9,
                b'd' | b'D' => z.right = 0.9,
                b's' | b'S' => self.down = !self.down,
                b'c' | b'C' => self.want_recalib = true,
                _ => {}
            }
        }
        if self.down {
            z.down = 0.9;
        }
        let s = ZoneActivity::new(z, self.t);
        self.t += DT;
        thread::sleep(Duration::from_secs_f64(DT));
        Some(s)
    }
}

impl Default for ManualSource {
    fn default() -> Self {
        Self::new()
    }
}

/// Non-blocking console input via the C runtime (UCRT).
mod console {
    #[cfg(windows)]
    extern "C" {
        fn _kbhit() -> i32;
        fn _getch() -> i32;
    }

    #[cfg(windows)]
    pub fn kbhit() -> bool {
        unsafe { _kbhit() != 0 }
    }

    #[cfg(windows)]
    pub fn getch() -> u8 {
        unsafe { _getch() as u8 }
    }

    #[cfg(not(windows))]
    pub fn kbhit() -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn getch() -> u8 {
        0
    }
}
