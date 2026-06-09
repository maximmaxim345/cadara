use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Sketch;

impl crate::operation::Operation for Sketch {
    type Change = ();
    fn apply_change(&mut self, _change: ()) {}
}
