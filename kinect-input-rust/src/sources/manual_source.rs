use std::f64::consts::PI;
use std::io;
use std::sync::mpsc::{self, Receiver};
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

#[derive(Debug)]
pub struct ManualSource {
    rx: Receiver<char>,
    first_call: bool,
    finished: bool,
    x: f64,
    t: f64,
    crouching: bool,
    jump_t: Option<f64>,
}

impl Default for ManualSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ManualSource {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut line = String::new();
                match stdin.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        for ch in line.chars().map(|ch| ch.to_ascii_lowercase()) {
                            let _ = tx.send(ch);
                        }
                    }
                }
            }
        });

        Self {
            rx,
            first_call: true,
            finished: false,
            x: 0.0,
            t: 0.0,
            crouching: false,
            jump_t: None,
        }
    }
}

impl Source for ManualSource {
    fn next(&mut self) -> Option<BodyState> {
        if self.finished {
            return None;
        }

        if self.first_call {
            self.first_call = false;
            println!("Steuerung: w + Enter = springen, s + Enter = ducken, a/d + Enter = links/rechts, q + Enter = Ende");
        }

        while let Ok(key) = self.rx.try_recv() {
            match key {
                'q' => {
                    self.finished = true;
                    return None;
                }
                'w' if self.jump_t.is_none() => self.jump_t = Some(self.t),
                's' => self.crouching = !self.crouching,
                'a' => self.x = (self.x - SIDE_X).max(-SIDE_X),
                'd' => self.x = (self.x + SIDE_X).min(SIDE_X),
                _ => {}
            }
        }

        let mut height = if self.crouching {
            CROUCH_HEIGHT
        } else {
            STAND_HEIGHT
        };

        if let Some(jump_t) = self.jump_t {
            let p = (self.t - jump_t) / 0.45;
            if p >= 1.0 {
                self.jump_t = None;
            } else {
                height = STAND_HEIGHT + JUMP_PEAK * (PI * p).sin();
            }
        }

        let out = BodyState::new(self.x, height, self.t);
        self.t += DT;
        thread::sleep(Duration::from_secs_f64(DT));
        Some(out)
    }
}
