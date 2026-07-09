//! Maps abstract detector events to concrete key presses (port of
//! pipeline.py). Deliberately separated from the detector so the detection
//! logic stays key-agnostic.

use crate::config;
use crate::detector::Event;
use crate::keysender::{Key, KeySender};

/// Turns an event into a key press. Returns the key for display purposes
/// (or None, e.g. on release).
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

/// Zone index (0..3) for highlighting an event in the overlay.
pub fn event_zone(event: Event) -> Option<usize> {
    match event {
        Event::TapLeft => Some(0),
        Event::TapRight => Some(1),
        Event::TapUp => Some(2),
        Event::HoldDownStart => Some(3),
        Event::HoldDownEnd => None,
    }
}
