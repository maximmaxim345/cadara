use crate::operation::ModelingOperation;
use module::DataSection;
use occara::{
    geom::{Point, Vector},
    shape::{Edge, Wire},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    name: String,
    operation: ModelingOperation,
    uuid: Uuid,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentData {
    steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Create {
    pub before: Option<Uuid>,
    pub operation: ModelingOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ModelingTransaction {
    Create(Create),
    Update,
    Delete,
}

impl DataSection for PersistentData {
    type Args = ModelingTransaction;

    fn apply(&mut self, args: Self::Args) {
        match args {
            ModelingTransaction::Create(c) => {
                let uuid = Uuid::new_v4();
                self.steps.push(Step {
                    name: "new operation".to_string(),
                    operation: c.operation,
                    uuid,
                });
            }
            ModelingTransaction::Update => todo!(),
            ModelingTransaction::Delete => todo!(),
        }
    }

    fn undo_history_name(_args: &Self::Args) -> String {
        String::new()
    }
}

impl PersistentData {
    #[must_use]
    pub fn shape(&self) -> occara::shape::Shape {
        let mut scale = 1.0;
        for _ in self
            .steps
            .iter()
            .filter(|s| matches!(s.operation, ModelingOperation::Grow))
        {
            scale *= 1.02;
        }

        for _ in self
            .steps
            .iter()
            .filter(|s| matches!(s.operation, ModelingOperation::Shrink))
        {
            scale /= 1.02;
        }
        let wire = {
            let p1 = Point::new(0.0, 0.0, 0.0);
            let p2 = Point::new(0.0, scale, 0.0);
            let p3 = Point::new(scale, scale, 0.0);
            let p4 = Point::new(scale, 0.0, 0.0);
            Wire::new(&[
                &Edge::line(&p1, &p2),
                &Edge::line(&p2, &p3),
                &Edge::line(&p3, &p4),
                &Edge::line(&p4, &p1),
            ])
        };
        let b = wire.face().extrude(&Vector::new(0.0, 0.0, scale));

        let mut f = b.fillet();
        for e in b.edges() {
            f.add(0.2, &e);
        }
        f.build()
    }
}
