use serde::{Deserialize, Serialize};

pub mod extrude;
pub mod sketch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ModelingOperation {
    Sketch(sketch::Sketch),
    Extrude(extrude::Extrude),
}
