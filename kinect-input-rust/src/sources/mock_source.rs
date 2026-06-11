use std::f64::consts::PI;
use std::thread;
use std::time::Duration;

use crate::body_state::BodyState;

use super::Source;

const FPS: f64 = 30.0;
const DT: f64 = 1.0 / FPS;
const STAND_HEIGHT: f64 = 1.75;
const CROUCH_HEIGHT: f64 = 1.30;
const JUMP_PEAK: f64 = 0.32;
const SIDE_X: f64 = 0.55;

#[derive(Debug, Clone)]
struct Step {
    message: Option<String>,
    x: f64,
    height: f64,
}

#[derive(Debug)]
pub struct MockSource {
    script: Vec<Step>,
    index: usize,
    t: f64,
    x: f64,
    pending_message: Option<String>,
    end_message: Option<String>,
    realtime: bool,
    rng: Lcg,
}

impl Default for MockSource {
    fn default() -> Self {
        Self::new(true)
    }
}

impl MockSource {
    pub fn new(realtime: bool) -> Self {
        let mut source = Self {
            script: Vec::new(),
            index: 0,
            t: 0.0,
            x: 0.0,
            pending_message: None,
            end_message: None,
            realtime,
            rng: Lcg::new(0x5eed_1234),
        };

        source.say(">> Simulation: Person steht still (Kalibrierung)...");
        source.hold(1.5, STAND_HEIGHT);
        source.say(">> Simulation: SPRUNG");
        source.jump(0.45);
        source.hold(1.0, STAND_HEIGHT);
        source.say(">> Simulation: Schritt nach LINKS");
        source.step_to(-SIDE_X, 0.35);
        source.hold(0.8, STAND_HEIGHT);
        source.say(">> Simulation: zurueck zur Mitte");
        source.step_to(0.0, 0.35);
        source.hold(0.8, STAND_HEIGHT);
        source.say(">> Simulation: DUCKEN (1.2s)");
        source.hold(1.2, CROUCH_HEIGHT);
        source.say(">> Simulation: wieder aufstehen");
        source.hold(1.0, STAND_HEIGHT);
        source.say(">> Simulation: Schritt nach RECHTS");
        source.step_to(SIDE_X, 0.35);
        source.hold(0.8, STAND_HEIGHT);
        source.say(">> Simulation: zurueck zur Mitte");
        source.step_to(0.0, 0.35);
        source.hold(0.5, STAND_HEIGHT);
        source.say(">> Simulation: noch ein SPRUNG");
        source.jump(0.45);
        source.hold(1.0, STAND_HEIGHT);
        source.end_message = Some(">> Simulation beendet.".to_string());

        source
    }

    fn say(&mut self, message: &str) {
        self.pending_message = Some(message.to_string());
    }

    fn push(&mut self, x: f64, height: f64) {
        let message = self.pending_message.take();
        let nx = x + self.noise();
        let nh = height + self.noise();
        self.script.push(Step {
            message,
            x: nx,
            height: nh,
        });
    }

    fn hold(&mut self, seconds: f64, height: f64) {
        let frames = (seconds * FPS) as usize;
        for _ in 0..frames {
            self.push(self.x, height);
        }
    }

    fn jump(&mut self, duration: f64) {
        let frames = (duration * FPS) as usize;
        for i in 0..frames {
            let p = i as f64 / (frames - 1) as f64;
            self.push(self.x, STAND_HEIGHT + JUMP_PEAK * (PI * p).sin());
        }
    }

    fn step_to(&mut self, target_x: f64, duration: f64) {
        let frames = (duration * FPS) as usize;
        let start = self.x;
        for i in 0..frames {
            self.x = start + (target_x - start) * (i + 1) as f64 / frames as f64;
            self.push(self.x, STAND_HEIGHT);
        }
    }

    fn noise(&mut self) -> f64 {
        self.rng.next_range(-0.01, 0.01)
    }
}

impl Source for MockSource {
    fn next(&mut self) -> Option<BodyState> {
        if self.index >= self.script.len() {
            if let Some(message) = self.end_message.take() {
                println!("{message}");
            }
            return None;
        }

        let step = &self.script[self.index];
        self.index += 1;

        if let Some(message) = &step.message {
            println!("{message}");
        }

        let out = BodyState::new(step.x, step.height, self.t);
        self.t += DT;

        if self.realtime {
            thread::sleep(Duration::from_secs_f64(DT));
        }

        Some(out)
    }
}

#[derive(Debug)]
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_range(&mut self, min: f64, max: f64) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let unit = ((self.state >> 11) as f64) / ((1u64 << 53) as f64);
        min + (max - min) * unit
    }
}
