use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Extrude;

impl crate::operation::Operation for Extrude {
    type Change = ();
    fn apply_change(&mut self, _change: ()) {}
}
