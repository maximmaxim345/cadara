use module::DataSection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedData {}

impl DataSection for SharedData {
    type Args = ();

    fn apply(&mut self, _args: Self::Args) {}

    fn undo_history_name(_args: &Self::Args) -> String {
        String::new()
    }
}
