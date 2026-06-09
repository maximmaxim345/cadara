use crate::operation::extrude::ExtrudeChange;
use crate::operation::fillet::FilletChange;
use crate::operation::sketch::{SketchChange, SketchPrimitive};
use crate::operation::{ModelingOperation, Operation};
use module::DataSection;
use occara::shape::{Compound, Shape};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub name: String,
    pub operation: ModelingOperation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentData {
    steps: Vec<Step>,
}

impl PersistentData {
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelingTransaction {
    Create(Create),
    Delete(Delete),
    Rename(Rename),
    Reorder(Reorder),
    EditSketch {
        step_id: Uuid,
        change: SketchChange,
    },
    EditExtrude {
        step_id: Uuid,
        change: ExtrudeChange,
    },
    EditFillet {
        step_id: Uuid,
        change: FilletChange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Create {
    pub id: Uuid,
    pub before: Option<Uuid>,
    pub name: String,
    pub operation: ModelingOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Delete {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rename {
    pub id: Uuid,
    pub new_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reorder {
    pub id: Uuid,
    pub before: Option<Uuid>,
}

impl DataSection for PersistentData {
    type Args = ModelingTransaction;

    fn apply(&mut self, args: Self::Args) {
        match args {
            ModelingTransaction::Create(c) => {
                if self.steps.iter().any(|s| s.id == c.id) {
                    return;
                }
                let pos = c
                    .before
                    .and_then(|a| self.steps.iter().position(|s| s.id == a))
                    .unwrap_or(self.steps.len());
                self.steps.insert(
                    pos,
                    Step {
                        id: c.id,
                        name: c.name,
                        operation: c.operation,
                    },
                );
            }
            ModelingTransaction::Delete(Delete { id }) => {
                self.steps.retain(|s| s.id != id);
            }
            ModelingTransaction::Rename(Rename { id, new_name }) => {
                if let Some(s) = self.steps.iter_mut().find(|s| s.id == id) {
                    s.name = new_name;
                }
            }
            ModelingTransaction::Reorder(Reorder { id, before }) => {
                let Some(from) = self.steps.iter().position(|s| s.id == id) else {
                    return;
                };
                let item = self.steps.remove(from);
                let to = before
                    .and_then(|a| self.steps.iter().position(|s| s.id == a))
                    .unwrap_or(self.steps.len());
                self.steps.insert(to, item);
            }
            ModelingTransaction::EditSketch { step_id, change } => {
                if let Some(s) = self.steps.iter_mut().find(|s| s.id == step_id) {
                    if let ModelingOperation::Sketch(ref mut op) = s.operation {
                        op.apply_change(change);
                    }
                }
            }
            ModelingTransaction::EditExtrude { step_id, change } => {
                if let Some(s) = self.steps.iter_mut().find(|s| s.id == step_id) {
                    if let ModelingOperation::Extrude(ref mut op) = s.operation {
                        op.apply_change(change);
                    }
                }
            }
            ModelingTransaction::EditFillet { step_id, change } => {
                if let Some(s) = self.steps.iter_mut().find(|s| s.id == step_id) {
                    if let ModelingOperation::Fillet(ref mut op) = s.operation {
                        op.apply_change(change);
                    }
                }
            }
        }
    }

    fn undo_history_name(args: &Self::Args) -> String {
        match args {
            ModelingTransaction::Create(_) => "Create".into(),
            ModelingTransaction::Delete(_) => "Delete".into(),
            ModelingTransaction::Rename(_) => "Rename".into(),
            ModelingTransaction::Reorder(_) => "Reorder".into(),
            ModelingTransaction::EditSketch { change, .. } => sketch_undo_name(change),
            ModelingTransaction::EditExtrude { change, .. } => extrude_undo_name(change),
            ModelingTransaction::EditFillet { change, .. } => fillet_undo_name(change),
        }
    }
}

impl PersistentData {
    /// Placeholder. The real walk lands in Task 8.
    #[must_use]
    pub fn shape(&self) -> Shape {
        let mut c = Compound::builder();
        c.build()
    }
}

fn sketch_undo_name(c: &SketchChange) -> String {
    match c {
        SketchChange::SetPlane(_) => "Set plane",
        SketchChange::AddPrimitive { primitive, .. } => match primitive {
            SketchPrimitive::Line(_) => "Add line",
            SketchPrimitive::Circle(_) => "Add circle",
            SketchPrimitive::Arc(_) => "Add arc",
        },
        SketchChange::DeletePrimitive { .. } => "Del primitive",
        SketchChange::ReorderPrimitive { .. } => "Move primitive",
        SketchChange::EditPrimitive { .. } => "Edit primitive",
    }
    .into()
}

fn extrude_undo_name(c: &ExtrudeChange) -> String {
    match c {
        ExtrudeChange::SetSketchId(_) => "Set sketch",
        ExtrudeChange::SetDepth(_) => "Set depth",
        ExtrudeChange::SetDirection(_) => "Set direction",
        ExtrudeChange::SetMode(_) => "Set mode",
    }
    .into()
}

fn fillet_undo_name(c: &FilletChange) -> String {
    match c {
        FilletChange::SetRadius(_) => "Set radius",
        FilletChange::SetTarget(_) => "Set target",
    }
    .into()
}
