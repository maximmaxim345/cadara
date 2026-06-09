use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extrude {
    pub sketch_id: Uuid,
    pub depth: f64,
    pub direction: ExtrudeDirection,
    pub mode: ExtrudeMode,
}

impl Eq for Extrude {}

impl Hash for Extrude {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sketch_id.hash(state);
        self.depth.to_bits().hash(state);
        self.direction.hash(state);
        self.mode.hash(state);
    }
}

impl Default for Extrude {
    fn default() -> Self {
        Self {
            sketch_id: Uuid::nil(),
            depth: 1.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtrudeDirection {
    #[default]
    Normal,
    Reversed,
    Symmetric,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtrudeMode {
    #[default]
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtrudeChange {
    SetSketchId(Uuid),
    SetDepth(f64),
    SetDirection(ExtrudeDirection),
    SetMode(ExtrudeMode),
}

impl Eq for ExtrudeChange {}

impl Hash for ExtrudeChange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::SetSketchId(id) => {
                0u8.hash(state);
                id.hash(state);
            }
            Self::SetDepth(d) => {
                1u8.hash(state);
                d.to_bits().hash(state);
            }
            Self::SetDirection(d) => {
                2u8.hash(state);
                d.hash(state);
            }
            Self::SetMode(m) => {
                3u8.hash(state);
                m.hash(state);
            }
        }
    }
}

impl crate::operation::Operation for Extrude {
    type Change = ExtrudeChange;

    fn apply_change(&mut self, change: ExtrudeChange) {
        match change {
            ExtrudeChange::SetSketchId(id) => self.sketch_id = id,
            ExtrudeChange::SetDepth(d) => self.depth = d,
            ExtrudeChange::SetDirection(d) => self.direction = d,
            ExtrudeChange::SetMode(m) => self.mode = m,
        }
    }
}
