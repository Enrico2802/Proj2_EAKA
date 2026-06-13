use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use kinect_input_rust::sources::{ManualSource, MockSource, Source};
use kinect_input_rust::{Gesture, GestureDetector, Key, KeySender};

#[derive(Debug, Clone, Copy)]
enum ActionKind {
    Tap,
    Press,
    Release,
}

#[derive(Debug, Clone, Copy)]
struct Action {
    kind: ActionKind,
    key: Key,
}

fn main() {
    let opts = parse_args().unwrap_or_else(|code| process::exit(code));

    let sender = KeySender::new(!opts.send);
    let mut detector = GestureDetector::default();
    let mut source: Box<dyn Source> = match opts.source.as_str() {
        "mock" => Box::new(MockSource::default()),
        "manual" => Box::new(ManualSource::default()),
        _ => unreachable!("parse_args validates source"),
    };

    if opts.send {
        println!("ECHTER Tastatur-Modus! Fokussiere jetzt das Zielfenster, z.B. Notepad oder das Spiel...");
        for i in (1..=3).rev() {
            println!("  Start in {i}...");
            thread::sleep(Duration::from_secs(1));
        }
    }

    println!("Kalibrierung laeuft - bitte still stehen...");
    let mut was_calibrated = false;

    {
        let _ctrl_guard = CtrlReleaseGuard { sender: &sender };

        while let Some(state) = source.next() {
            let events = detector.update(state);

            if detector.calibrated() && !was_calibrated {
                was_calibrated = true;
                println!(
                    "Kalibriert: Baseline-Hoehe = {:.2}, Baseline-x = {:+.2}\n",
                    detector.baseline_height(),
                    detector.baseline_x()
                );
            }

            for gesture in events {
                let action = action_for(gesture);
                println!(
                    "[{:6.2}s] GESTE: {:<13} -> {} '{}'",
                    state.t,
                    gesture,
                    action_kind_str(action.kind),
                    action.key
                );

                let result = match action.kind {
                    ActionKind::Tap => sender.tap(action.key),
                    ActionKind::Press => sender.press(action.key),
                    ActionKind::Release => sender.release(action.key),
                };

                if let Err(err) = result {
                    eprintln!("      WARNUNG: Tastatur-Event fehlgeschlagen fuer {}: {err}", action.key);
                }
            }
        }
    }

    println!("\nFertig.");
}

#[derive(Debug)]
struct Options {
    source: String,
    send: bool,
}

fn parse_args() -> Result<Options, i32> {
    let args: Vec<String> = env::args().collect();
    let mut source = "mock".to_string();
    let mut send = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--send" => send = true,
            "--source" if i + 1 < args.len() => {
                i += 1;
                source = args[i].clone();
            }
            _ => return Err(usage(&args[0])),
        }
        i += 1;
    }

    if source != "mock" && source != "manual" {
        return Err(usage(&args[0]));
    }

    Ok(Options { source, send })
}

fn usage(prog: &str) -> i32 {
    eprintln!("Verwendung: {prog} [--source mock|manual] [--send]");
    eprintln!("  --source  Datenquelle: mock = Drehbuch-Simulation, manual = per w/a/s/d + Enter steuern");
    eprintln!("  --send    Tasten WIRKLICH senden (Standard: Dry-Run, nur Logging)");
    2
}

fn action_for(gesture: Gesture) -> Action {
    match gesture {
        Gesture::Jump => Action {
            kind: ActionKind::Tap,
            key: Key::Space,
        },
        Gesture::LaneLeft => Action {
            kind: ActionKind::Tap,
            key: Key::A,
        },
        Gesture::LaneRight => Action {
            kind: ActionKind::Tap,
            key: Key::D,
        },
        Gesture::CrouchStart => Action {
            kind: ActionKind::Press,
            key: Key::Ctrl,
        },
        Gesture::CrouchEnd => Action {
            kind: ActionKind::Release,
            key: Key::Ctrl,
        },
    }
}

fn action_kind_str(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Tap => "tap",
        ActionKind::Press => "press",
        ActionKind::Release => "release",
    }
}

struct CtrlReleaseGuard<'a> {
    sender: &'a KeySender,
}

impl Drop for CtrlReleaseGuard<'_> {
    fn drop(&mut self) {
        let _ = self.sender.release(Key::Ctrl);
    }
}
