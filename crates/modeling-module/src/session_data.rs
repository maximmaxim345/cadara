use module::{DataTransaction, ReversibleDataTransaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionData {}

impl DataTransaction for SessionData {
    type Args = ();
    type Error = ();
    type Output = ();

    fn apply(&mut self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn undo_history_name(_args: &Self::Args) -> String {
        String::new()
    }
}

impl ReversibleDataTransaction for SessionData {
    type UndoData = ();

    fn apply(&mut self, _args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error> {
        Ok(((), ()))
    }

    fn undo(&mut self, _undo_data: Self::UndoData) {}
}
