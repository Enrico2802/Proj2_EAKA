#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    pub x: f64,
    pub height: f64,
    pub t: f64,
}

impl BodyState {
    pub fn new(x: f64, height: f64, t: f64) -> Self {
        Self { x, height, t }
    }
}
