#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use module::{DataTransaction, Module, ReversibleDataTransaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct ModelingModule {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSection {}

impl DataTransaction for DataSection {
    type Args = ();

    type Error = ();

    type Output = ();

    fn apply(&mut self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn undo_history_name(_args: &Self::Args) -> String {
        "nothing".to_string()
    }
}

impl ReversibleDataTransaction for DataSection {
    type UndoData = ();

    fn apply(&mut self, _args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error> {
        Ok(((), ()))
    }

    fn undo(&mut self, _undo_data: Self::UndoData) {}
}

impl Module for ModelingModule {
    type PersistentData = DataSection;

    type PersistentUserData = DataSection;

    type SessionData = DataSection;

    type SharedData = DataSection;

    fn name() -> String {
        "Modeling".to_string()
    }

    fn uuid() -> uuid::Uuid {
        uuid::Uuid::parse_str("04d338d9-b7a9-4f5a-b04d-724466f4058f").expect("static UUID")
    }
}
