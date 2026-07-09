//! Webcam motion control -> keyboard. CLI entry point (port of main.py).
//!
//!   cargo run                              # startup GUI (source selection) -> webcam
//!   cargo run -- --source webcam --no-gui  # webcam directly, without startup GUI
//!   cargo run -- --source mock             # scripted mock, dry run
//!   cargo run -- --source manual           # control via w/a/s/d, q = quit
//!   ... --send                             # real key presses (with --no-gui/mock/manual)
//!   ... --no-flow                          # disable the optical-flow direction gate
//!   ... --enter 0.2 / --exit 0.1           # override the threshold for ALL zones

use std::thread;
use std::time::Duration;

use webcam_rust::config;
use webcam_rust::detector::{Event, GestureDetector};
use webcam_rust::keysender::KeySender;
use webcam_rust::launcher;
use webcam_rust::overlay::{Action, Overlay};
use webcam_rust::pipeline;
use webcam_rust::sources::{ManualSource, MockSource};
use webcam_rust::webcam_source::WebcamGridSource;

struct Args {
    source: String,
    send: bool,
    no_gui: bool,
    camera: i32,
    show: bool,
    mirror: bool,
    /// Overrides the enter threshold of ALL four zones (None = per-zone
    /// values from config::ZONE_ENTER_RATIOS).
    enter: Option<f64>,
    exit: Option<f64>,
    flow: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        source: "webcam".into(),
        send: false,
        no_gui: false,
        camera: config::CAMERA_INDEX,
        show: true,
        mirror: config::MIRROR,
        enter: None,
        exit: None,
        flow: config::FLOW_ENABLED,
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--source" => { i += 1; if i < argv.len() { a.source = argv[i].clone(); } }
            "--send" => a.send = true,
            "--no-gui" => a.no_gui = true,
            "--camera" => { i += 1; if i < argv.len() { a.camera = argv[i].parse().unwrap_or(a.camera); } }
            "--no-show" => a.show = false,
            "--show" => a.show = true,
            "--no-mirror" => a.mirror = false,
            "--mirror" => a.mirror = true,
            "--enter" => { i += 1; if i < argv.len() { a.enter = argv[i].parse().ok().or(a.enter); } }
            "--exit" => { i += 1; if i < argv.len() { a.exit = argv[i].parse().ok().or(a.exit); } }
            "--no-flow" => a.flow = false,
            other => eprintln!("Unbekanntes Argument: {other}"),
        }
        i += 1;
    }
    a
}

fn make_detector(a: &Args) -> GestureDetector {
    let mut cfg = config::detector_config();
    if let Some(e) = a.enter {
        cfg.enter_ratio = [e; 4];
    }
    if let Some(e) = a.exit {
        cfg.exit_ratio = [e; 4];
    }
    GestureDetector::new(cfg)
}

fn countdown() {
    println!("ECHTER Tastatur-Modus! Fokussiere jetzt das Zielfenster...");
    for i in (1..=3).rev() {
        println!("  Start in {i}...");
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let mut args = parse_args();

    // Startup GUI before the proof screen: camera selection + mode (real/demo).
    if args.source == "webcam" && !args.no_gui {
        match launcher::run_launcher(args.camera) {
            Ok(Some(cfg)) => {
                args.camera = cfg.camera;
                args.send = cfg.send;
            }
            Ok(None) => {
                println!("Abgebrochen.");
                return;
            }
            Err(e) => {
                eprintln!("Launcher-Fehler: {e}");
                return;
            }
        }
    }

    let mut detector = make_detector(&args);
    let mut sender = KeySender::new(args.send);

    if args.send && args.source != "webcam" {
        countdown();
    }
    println!("Kalibrierung laeuft - bitte still stehen / Ruhe halten...");

    match args.source.as_str() {
        "webcam" => run_webcam(&args, &mut detector, &mut sender),
        "manual" => run_manual(&mut detector, &mut sender),
        _ => run_mock(&mut detector, &mut sender),
    }

    sender.release_all();
    println!("\nFertig.");
}

fn handle_events(events: &[webcam_rust::detector::Event], sender: &mut KeySender, t: f64)
    -> (Option<webcam_rust::keysender::Key>, Option<usize>)
{
    let mut current_key = None;
    let mut triggered = None;
    for &ev in events {
        if let Some(k) = pipeline::dispatch(ev, sender) {
            current_key = Some(k);
        }
        if let Some(z) = pipeline::event_zone(ev) {
            triggered = Some(z);
        }
        println!("[{t:6.2}s] {ev}");
    }
    (current_key, triggered)
}

fn run_mock(detector: &mut GestureDetector, sender: &mut KeySender) {
    let mut src = MockSource::new(true);
    while let Some(s) = src.next() {
        let events = detector.update(s);
        handle_events(&events, sender, s.t);
    }
}

fn run_manual(detector: &mut GestureDetector, sender: &mut KeySender) {
    let mut src = ManualSource::new();
    loop {
        let Some(s) = src.next() else { break };
        if src.want_recalib {
            detector.start_recalibration();
            println!(">> Neu-Kalibrierung gestartet...");
        }
        let events = detector.update(s);
        handle_events(&events, sender, s.t);
    }
}

fn run_webcam(args: &Args, detector: &mut GestureDetector, sender: &mut KeySender) {
    let mut src = match WebcamGridSource::new(args.camera, args.mirror) {
        Ok(s) => s,
        Err(e) => { eprintln!("Kamera-Fehler: {e}"); return; }
    };
    src.use_flow = args.flow;
    let mut overlay = if args.show {
        match Overlay::new() {
            Ok(o) => Some(o),
            Err(e) => { eprintln!("Overlay-Fehler: {e}"); None }
        }
    } else {
        None
    };

    loop {
        let s = match src.next() {
            Ok(Some(s)) => s,
            Ok(None) => continue,
            Err(e) => { eprintln!("Frame-Fehler: {e}"); break; }
        };
        // The source auto-recalibrated (lighting change) -> relearn the
        // detector baseline as well; this frame is unusable anyway.
        if src.want_recalib {
            detector.start_recalibration();
            sender.release_all();
            println!(">> Neu-Kalibrierung gestartet (automatisch)...");
            continue;
        }
        let mut events = detector.update(s);
        // During a suspected scene change do not trigger new keys - only
        // releasing an already held S is still allowed.
        if src.scene_suspect() {
            events.retain(|e| *e == Event::HoldDownEnd);
        }
        let (current_key, triggered) = handle_events(&events, sender, s.t);

        if let Some(ov) = overlay.as_mut() {
            match ov.show(&src.frame, &src.active, detector.calibrated(),
                          current_key, triggered, sender.send_enabled()) {
                Ok(Action::Quit) => break,
                Ok(Action::Recalibrate) => {
                    detector.start_recalibration();
                    src.recalibrate();
                    println!(">> Neu-Kalibrierung gestartet...");
                }
                Ok(Action::ToggleSend) => sender.set_send(!sender.send_enabled()),
                Ok(Action::None) => {}
                Err(e) => eprintln!("Overlay-Fehler: {e}"),
            }
        }
    }
    if let Some(ov) = overlay.as_ref() {
        ov.close();
    }
}
