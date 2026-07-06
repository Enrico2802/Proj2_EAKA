//! Webcam-Bewegungssteuerung -> Tastatur. CLI-Einstieg (Port von main.py).
//!
//!   cargo run                              # Start-GUI (Quellenauswahl) -> Webcam
//!   cargo run -- --source webcam --no-gui  # Webcam direkt, ohne Start-GUI
//!   cargo run -- --source mock             # Mock-Drehbuch, Dry-Run
//!   cargo run -- --source manual           # w/a/s/d steuern, q=Ende
//!   ... --send                             # echte Tasten (bei --no-gui/mock/manual)

use std::thread;
use std::time::Duration;

use webcam_rust::config;
use webcam_rust::detector::GestureDetector;
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
    enter: f64,
    exit: f64,
}

fn parse_args() -> Args {
    let mut a = Args {
        source: "webcam".into(),
        send: false,
        no_gui: false,
        camera: config::CAMERA_INDEX,
        show: true,
        mirror: config::MIRROR,
        enter: config::ZONE_ENTER_RATIO,
        exit: config::ZONE_EXIT_RATIO,
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
            "--enter" => { i += 1; if i < argv.len() { a.enter = argv[i].parse().unwrap_or(a.enter); } }
            "--exit" => { i += 1; if i < argv.len() { a.exit = argv[i].parse().unwrap_or(a.exit); } }
            other => eprintln!("Unbekanntes Argument: {other}"),
        }
        i += 1;
    }
    a
}

fn make_detector(a: &Args) -> GestureDetector {
    let mut cfg = config::detector_config();
    cfg.enter_ratio = a.enter;
    cfg.exit_ratio = a.exit;
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

    // Start-GUI vor dem Beweis-Screen: Quellenauswahl + Modus (Real/Demo).
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
        let events = detector.update(s);
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
