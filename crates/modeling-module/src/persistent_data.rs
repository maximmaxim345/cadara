use crate::operation::extrude::{Extrude, ExtrudeChange, ExtrudeDirection, ExtrudeMode};
use crate::operation::fillet::{Fillet, FilletChange, FilletTarget};
use crate::operation::sketch::{Plane, Point2D, Sketch, SketchChange, SketchPrimitive};
use crate::operation::{ModelingOperation, Operation};
use module::DataSection;
use occara::geom::{Direction, Point, Vector};
use occara::shape::{Compound, Edge, Face, Shape, Wire};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    #[must_use]
    pub fn shape(&self) -> Shape {
        let mut state = WalkState::new();
        for step in &self.steps {
            state.apply(step);
        }
        state.body
    }
}

struct WalkState {
    body: Shape,
    sketch_outputs: HashMap<Uuid, (Face, Plane)>,
}

impl WalkState {
    fn new() -> Self {
        Self {
            body: Compound::builder().build(),
            sketch_outputs: HashMap::new(),
        }
    }

    fn apply(&mut self, step: &Step) {
        match &step.operation {
            ModelingOperation::Sketch(s) => self.apply_sketch(step.id, s),
            ModelingOperation::Extrude(e) => self.apply_extrude(e),
            ModelingOperation::Fillet(f) => self.apply_fillet(f),
        }
    }

    fn apply_sketch(&mut self, step_id: Uuid, sketch: &Sketch) {
        let edges: Vec<Edge> = sketch
            .primitives
            .iter()
            .map(|(_, p)| build_edge(p, sketch.plane))
            .collect();
        if edges.is_empty() {
            return;
        }
        let edge_refs: Vec<&Edge> = edges.iter().collect();
        let trait_refs: Vec<&dyn occara::shape::AddableToWire> = edge_refs
            .iter()
            .map(|e| *e as &dyn occara::shape::AddableToWire)
            .collect();
        let wire = Wire::new(&trait_refs);
        let face = wire.face();
        self.sketch_outputs.insert(step_id, (face, sketch.plane));
    }

    fn apply_extrude(&mut self, e: &Extrude) {
        let Some((face, plane)) = self.sketch_outputs.get(&e.sketch_id) else {
            return;
        };
        let normal = plane_normal(*plane);
        let extrusion = match e.direction {
            ExtrudeDirection::Normal => face.extrude(&scaled_vector(&normal, e.depth)),
            ExtrudeDirection::Reversed => face.extrude(&scaled_vector(&normal, -e.depth)),
            ExtrudeDirection::Symmetric => {
                let half = e.depth / 2.0;
                let up = face.extrude(&scaled_vector(&normal, half));
                let down = face.extrude(&scaled_vector(&normal, -half));
                up.fuse(&down)
            }
        };
        self.body = match e.mode {
            ExtrudeMode::Add => {
                if body_is_empty(&self.body) {
                    extrusion
                } else {
                    self.body.fuse(&extrusion)
                }
            }
            ExtrudeMode::Subtract => {
                if body_is_empty(&self.body) {
                    return;
                }
                self.body.subtract(&extrusion)
            }
        };
    }

    fn apply_fillet(&mut self, f: &Fillet) {
        let edges: Vec<Edge> = self.body.edges().collect();
        let chosen: Vec<Edge> = match &f.target {
            FilletTarget::WholeBody => edges,
            FilletTarget::Face(face_ref) => {
                let Some(face) = self.body.faces().nth(face_ref.index) else {
                    return;
                };
                face.edges().collect()
            }
            FilletTarget::Edges(refs) => refs
                .iter()
                .filter_map(|er| edges.get(er.index).cloned())
                .collect(),
        };
        if chosen.is_empty() {
            return;
        }
        let mut builder = self.body.fillet();
        for edge in &chosen {
            builder.add(f.radius, edge);
        }
        if let Ok(filleted) = builder.build() {
            self.body = filleted;
        }
    }
}

fn body_is_empty(shape: &Shape) -> bool {
    shape.faces().next().is_none()
}

fn plane_normal(plane: Plane) -> Direction {
    match plane {
        Plane::XY => Direction::z(),
        Plane::YZ => Direction::x(),
        Plane::XZ => Direction::y(),
    }
}

fn scaled_vector(d: &Direction, s: f64) -> Vector {
    let (dx, dy, dz) = d.get_components();
    Vector::new(dx * s, dy * s, dz * s)
}

fn lift(plane: Plane, p: Point2D) -> Point {
    match plane {
        Plane::XY => Point::new(p.x, p.y, 0.0),
        Plane::YZ => Point::new(0.0, p.x, p.y),
        Plane::XZ => Point::new(p.x, 0.0, p.y),
    }
}

fn build_edge(prim: &SketchPrimitive, plane: Plane) -> Edge {
    match prim {
        SketchPrimitive::Line(l) => {
            let a = lift(plane, l.from);
            let b = lift(plane, l.to);
            Edge::line(&a, &b)
        }
        SketchPrimitive::Circle(c) => {
            let center = lift(plane, c.center);
            let normal = plane_normal(plane);
            Edge::circle(&center, &normal, c.radius)
        }
        SketchPrimitive::Arc(a) => {
            let p1 = lift(plane, a.from);
            let p2 = lift(plane, a.through);
            let p3 = lift(plane, a.to);
            Edge::arc_of_circle(&p1, &p2, &p3)
        }
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
