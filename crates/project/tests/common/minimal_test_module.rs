// A second minimal test module, to test multiple different modules
use module::{DataSection, Module};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimalTestModule {}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MinimalTestDataSection {
    pub num: i32,
}

impl DataSection for MinimalTestDataSection {
    type Args = i32;

    fn apply(&mut self, args: Self::Args) {
        self.num = args;
    }

    fn undo_history_name(args: &Self::Args) -> String {
        format!("Set num to {args}")
    }
}

impl Module for MinimalTestModule {
    type PersistentData = MinimalTestDataSection;
    type PersistentUserData = MinimalTestDataSection;
    type SessionData = MinimalTestDataSection;
    type SharedData = MinimalTestDataSection;

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
    let mut data_section = MinimalTestDataSection::default();
    assert_eq!(data_section.num, 0);
    DataSection::apply(&mut data_section, 4);
    assert_eq!(data_section.num, 4);
    DataSection::apply(&mut data_section, 40);
    assert_eq!(data_section.num, 40);
}
