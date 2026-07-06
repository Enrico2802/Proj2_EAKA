//! Webcam-Bewegungssteuerung -> Tastatur (Rust-Port).
//!
//! Kamerafreie Kernlogik (testbar ohne Hardware). Die OpenCV-abhaengigen Teile
//! (Webcam-Quelle, Overlay) liegen in eigenen Modulen und werden vom Binary
//! eingebunden.

pub mod config;
pub mod detector;
pub mod keysender;
pub mod launcher;
pub mod overlay;
pub mod pipeline;
pub mod sources;
pub mod webcam_source;
pub mod zones;
