use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fillet {
    pub radius: f64,
    pub target: FilletTarget,
}

impl Eq for Fillet {}

impl Hash for Fillet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.radius.to_bits().hash(state);
        self.target.hash(state);
    }
}

impl Default for Fillet {
    fn default() -> Self {
        Self {
            radius: 0.1,
            target: FilletTarget::WholeBody,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilletTarget {
    WholeBody,
    Face(FaceRef),
    Edges(Vec<EdgeRef>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceRef {
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeRef {
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilletChange {
    SetRadius(f64),
    SetTarget(FilletTarget),
}

impl Eq for FilletChange {}

impl Hash for FilletChange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::SetRadius(r) => {
                0u8.hash(state);
                r.to_bits().hash(state);
            }
            Self::SetTarget(t) => {
                1u8.hash(state);
                t.hash(state);
            }
        }
    }
}

impl crate::operation::Operation for Fillet {
    type Change = FilletChange;

    fn apply_change(&mut self, change: FilletChange) {
        match change {
            FilletChange::SetRadius(r) => self.radius = r,
            FilletChange::SetTarget(t) => self.target = t,
        }
    }
}
