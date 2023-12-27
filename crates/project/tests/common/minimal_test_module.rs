// A second minimal test module, to test multiple different modules
use document::Module;
use project::transaction::DocumentTransaction;
use project::*;
use serde::{Deserialize, Serialize};
use transaction::ReversibleDocumentTransaction;
use uuid::Uuid;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct MinimalTestModule {}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TestDataSection {
    pub num: i32,
}

impl DocumentTransaction for TestDataSection {
    type Args = i32;
    type Error = ();
    type Output = ();

    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use the undoable transaction to implement this
        <Self as ReversibleDocumentTransaction>::apply(self, args)
            .map(|(output, _undo_data)| output)
    }

    fn undo_history_name(args: &Self::Args) -> String {
        format!("Set num to {args}")
    }
}

impl ReversibleDocumentTransaction for TestDataSection {
    type UndoData = i32;
    fn apply(&mut self, args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error> {
        let old_num = self.num;
        self.num = args;
        Ok(((), old_num))
    }
    fn undo(&mut self, undo_data: Self::UndoData) {
        self.num = undo_data;
    }
}

impl Module for MinimalTestModule {
    type DocumentData = TestDataSection;
    type UserData = TestDataSection;
    type SessionData = TestDataSection;
    type SharedData = TestDataSection;

    fn name() -> String {
        "A Minimal Test Module".to_string()
    }
    fn uuid() -> Uuid {
        Uuid::parse_str("fbf7651c-6152-4068-8df5-15e41052a8f1").unwrap()
    }
}

// This is a test of this test module, we do not mark it as a test here,
// since otherwise it will be run every time another test includes this module
#[allow(dead_code)]
pub fn test_minimal_test_module() {
    let mut data_section = TestDataSection::default();
    assert_eq!(data_section.num, 0);
    assert!(DocumentTransaction::apply(&mut data_section, 4).is_ok());
    assert_eq!(data_section.num, 4);
    let undo_data = ReversibleDocumentTransaction::apply(&mut data_section, 40)
        .unwrap()
        .1;
    assert_eq!(data_section.num, 40);
    ReversibleDocumentTransaction::undo(&mut data_section, undo_data);
    assert_eq!(data_section.num, 4);
}
