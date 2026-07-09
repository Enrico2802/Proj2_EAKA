//! Webcam motion control -> keyboard (Rust port).
//!
//! Camera-free core logic (testable without hardware). The OpenCV-dependent
//! parts (webcam source, overlay) live in separate modules and are wired up
//! by the binary.

pub mod config;
pub mod detector;
pub mod keysender;
pub mod launcher;
pub mod overlay;
pub mod pipeline;
pub mod sources;
pub mod webcam_source;
pub mod zones;
