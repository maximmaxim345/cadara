use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Eq for Point2D {}

impl Hash for Point2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sketch {
    pub plane: Plane,
    pub primitives: Vec<(Uuid, SketchPrimitive)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plane {
    #[default]
    XY,
    YZ,
    XZ,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SketchPrimitive {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub from: Point2D,
    pub to: Point2D,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2D,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub from: Point2D,
    pub through: Point2D,
    pub to: Point2D,
}

impl Eq for Line {}
impl Eq for Circle {}
impl Eq for Arc {}

impl Hash for Line {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl Hash for Circle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.center.hash(state);
        self.radius.to_bits().hash(state);
    }
}

impl Hash for Arc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.through.hash(state);
        self.to.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SketchChange {
    SetPlane(Plane),
    AddPrimitive {
        id: Uuid,
        before: Option<Uuid>,
        primitive: SketchPrimitive,
    },
    DeletePrimitive {
        id: Uuid,
    },
    ReorderPrimitive {
        id: Uuid,
        before: Option<Uuid>,
    },
    EditPrimitive {
        id: Uuid,
        change: PrimitiveChange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveChange {
    Line(LineChange),
    Circle(CircleChange),
    Arc(ArcChange),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineChange {
    SetFrom(Point2D),
    SetTo(Point2D),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircleChange {
    SetCenter(Point2D),
    SetRadius(f64),
}

impl Eq for CircleChange {}

impl Hash for CircleChange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::SetCenter(p) => {
                0u8.hash(state);
                p.hash(state);
            }
            Self::SetRadius(r) => {
                1u8.hash(state);
                r.to_bits().hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArcChange {
    SetFrom(Point2D),
    SetThrough(Point2D),
    SetTo(Point2D),
}

impl crate::operation::Operation for Sketch {
    type Change = SketchChange;

    fn apply_change(&mut self, change: SketchChange) {
        match change {
            SketchChange::SetPlane(p) => self.plane = p,
            SketchChange::AddPrimitive {
                id,
                before,
                primitive,
            } => {
                if self.primitives.iter().any(|(i, _)| *i == id) {
                    return;
                }
                let pos = before
                    .and_then(|a| self.primitives.iter().position(|(i, _)| *i == a))
                    .unwrap_or(self.primitives.len());
                self.primitives.insert(pos, (id, primitive));
            }
            SketchChange::DeletePrimitive { id } => {
                self.primitives.retain(|(i, _)| *i != id);
            }
            SketchChange::ReorderPrimitive { id, before } => {
                let Some(from) = self.primitives.iter().position(|(i, _)| *i == id) else {
                    return;
                };
                let item = self.primitives.remove(from);
                let to = before
                    .and_then(|a| self.primitives.iter().position(|(i, _)| *i == a))
                    .unwrap_or(self.primitives.len());
                self.primitives.insert(to, item);
            }
            SketchChange::EditPrimitive { id, change } => {
                let Some(slot) = self.primitives.iter_mut().find(|(i, _)| *i == id) else {
                    return;
                };
                apply_primitive_change(&mut slot.1, change);
            }
        }
    }
}

const fn apply_primitive_change(prim: &mut SketchPrimitive, change: PrimitiveChange) {
    match (prim, change) {
        (SketchPrimitive::Line(l), PrimitiveChange::Line(c)) => match c {
            LineChange::SetFrom(p) => l.from = p,
            LineChange::SetTo(p) => l.to = p,
        },
        (SketchPrimitive::Circle(c), PrimitiveChange::Circle(ch)) => match ch {
            CircleChange::SetCenter(p) => c.center = p,
            CircleChange::SetRadius(r) => c.radius = r,
        },
        (SketchPrimitive::Arc(a), PrimitiveChange::Arc(ch)) => match ch {
            ArcChange::SetFrom(p) => a.from = p,
            ArcChange::SetThrough(p) => a.through = p,
            ArcChange::SetTo(p) => a.to = p,
        },
        _ => {}
    }
}
