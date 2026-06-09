use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

pub mod extrude;
pub mod fillet;
pub mod sketch;

pub trait Operation:
    Clone + Debug + Eq + Hash + Serialize + for<'de> Deserialize<'de> + Send + Sync
{
    type Change: Clone + Debug + Eq + Hash + Serialize + for<'de> Deserialize<'de> + Send + Sync;
    fn apply_change(&mut self, change: Self::Change);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ModelingOperation {
    Sketch(sketch::Sketch),
    Extrude(extrude::Extrude),
    Fillet(fillet::Fillet),
    Grow,
    Shrink,
}
