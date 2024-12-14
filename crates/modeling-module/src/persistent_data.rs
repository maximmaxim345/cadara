use crate::operation::ModelingOperation;
use module::{DataSection, ReversibleDataTransaction};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ModelingTransactionOutput {
    uuid: Uuid,
}

impl DataSection for PersistentData {
    type Args = ModelingTransaction;
    type Error = ();
    type Output = ModelingTransactionOutput;

    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        ReversibleDataTransaction::apply(self, args).map(|a| a.0)
    }

    fn undo_history_name(_args: &Self::Args) -> String {
        String::new()
    }
}

impl ReversibleDataTransaction for PersistentData {
    type UndoData = ();

    fn apply(&mut self, args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error> {
        match args {
            ModelingTransaction::Create(c) => {
                let uuid = Uuid::new_v4();
                self.steps.push(Step {
                    name: "new operation".to_string(),
                    operation: c.operation,
                    uuid,
                });
                Ok((ModelingTransactionOutput { uuid }, ()))
            }
            ModelingTransaction::Update => todo!(),
            ModelingTransaction::Delete => todo!(),
        }
    }

    fn undo(&mut self, _undo_data: Self::UndoData) {
        unimplemented!("not supported")
    }
}

impl PersistentData {
    #[must_use]
    pub fn shape(&self) -> occara::shape::Shape {
        let wire = {
            let p1 = Point::new(0.0, 0.0, 0.0);
            let p2 = Point::new(0.0, 1.0, 0.0);
            let p3 = Point::new(1.0, 1.0, 0.0);
            let p4 = Point::new(1.0, 0.0, 0.0);
            Wire::new(&[
                &Edge::line(&p1, &p2),
                &Edge::line(&p2, &p3),
                &Edge::line(&p3, &p4),
                &Edge::line(&p4, &p1),
            ])
        };
        let b = wire.face().extrude(&Vector::new(0.0, 0.0, 1.0));

        let mut f = b.fillet();
        for e in b.edges() {
            f.add(0.2, &e);
        }
        f.build()
    }
}
