mod manual_source;
mod mock_source;

pub use manual_source::ManualSource;
pub use mock_source::MockSource;

use crate::body_state::BodyState;

pub trait Source {
    fn next(&mut self) -> Option<BodyState>;
}
