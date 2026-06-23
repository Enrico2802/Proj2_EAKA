//! Mapping abstrakter Detector-Events auf konkrete Tastendruecke (Port von
//! pipeline.py). Bewusst getrennt vom Detector, damit die Erkennungslogik
//! tastenneutral bleibt.

use crate::config;
use crate::detector::Event;
use crate::keysender::{Key, KeySender};

/// Setzt ein Event in einen Tastendruck um. Gibt die Taste fuer die Anzeige
/// zurueck (oder None, z.B. beim Loslassen).
pub fn dispatch(event: Event, sender: &mut KeySender) -> Option<Key> {
    match event {
        Event::TapLeft => {
            sender.tap(config::KEY_LEFT);
            Some(config::KEY_LEFT)
        }
        Event::TapRight => {
            sender.tap(config::KEY_RIGHT);
            Some(config::KEY_RIGHT)
        }
        Event::TapUp => {
            sender.tap(config::KEY_UP);
            Some(config::KEY_UP)
        }
        Event::HoldDownStart => {
            sender.hold(config::KEY_DOWN, true);
            Some(config::KEY_DOWN)
        }
        Event::HoldDownEnd => {
            sender.hold(config::KEY_DOWN, false);
            None
        }
    }
}

/// Zone-Index (0..3) fuer die Overlay-Hervorhebung eines Events.
pub fn event_zone(event: Event) -> Option<usize> {
    match event {
        Event::TapLeft => Some(0),
        Event::TapRight => Some(1),
        Event::TapUp => Some(2),
        Event::HoldDownStart => Some(3),
        Event::HoldDownEnd => None,
    }
}
