use module::{DataSection, Module};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Skipped,
    Applied,
}

pub type TransactionLog = Vec<(TransactionStatus, <TestDataSection as DataSection>::Args)>;

lazy_static! {
    static ref GLOBAL_TRANSACTION_LOG: Arc<RwLock<HashMap<Uuid, TransactionLog>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub fn clear_transaction_log(uuid: Uuid) {
    GLOBAL_TRANSACTION_LOG.write().unwrap().remove(&uuid);
}

pub fn get_transaction_log(uuid: Uuid) -> TransactionLog {
    match GLOBAL_TRANSACTION_LOG.read().unwrap().get(&uuid) {
        Some(v) => v.clone(),
        None => vec![],
    }
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize)]
pub struct TestModule {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestDataSection {
    pub single_word: String,
    pub odd_number: i32,
    pub logging_uuid: Uuid,
}
// PartialEq should ignore the logging_uuid field
impl PartialEq for TestDataSection {
    fn eq(&self, other: &Self) -> bool {
        self.single_word == other.single_word && self.odd_number == other.odd_number
    }
}

impl Default for TestDataSection {
    fn default() -> Self {
        Self {
            single_word: "default".to_string(),
            odd_number: 1,
            logging_uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum TestTransaction {
    SetWord(String),
    SetNumber(i32),
    ResetIfNumberIsOver100,
}

impl DataSection for TestDataSection {
    type Args = TestTransaction;
    fn apply(&mut self, args: Self::Args) {
        let result = match args.clone() {
            TestTransaction::SetWord(word) => {
                if word.contains(' ') {
                    TransactionStatus::Skipped
                } else {
                    self.single_word.clone_from(&word);
                    TransactionStatus::Applied
                }
            }
            TestTransaction::SetNumber(number) => {
                if number % 2 == 0 {
                    TransactionStatus::Skipped
                } else {
                    self.odd_number = number;
                    TransactionStatus::Applied
                }
            }
            TestTransaction::ResetIfNumberIsOver100 => {
                if self.odd_number > 100 {
                    self.odd_number = 0;
                    TransactionStatus::Applied
                } else {
                    TransactionStatus::Skipped
                }
            }
        };
        let mut map = GLOBAL_TRANSACTION_LOG.write().unwrap();
        let log = map.entry(self.logging_uuid).or_default();

        // log the result
        log.push((result, args.clone()));
    }

    fn undo_history_name(args: &Self::Args) -> String {
        match args {
            TestTransaction::SetWord(word) => format!("Set word to {word}"),
            TestTransaction::SetNumber(number) => format!("Set number to {number}"),
            TestTransaction::ResetIfNumberIsOver100 => "Fail if number is over 100".to_string(),
        }
    }
}

impl Module for TestModule {
    type PersistentData = TestDataSection;
    type PersistentUserData = TestDataSection;
    type SessionData = TestDataSection;
    type SharedData = TestDataSection;

    fn name() -> String {
        "Test Module".to_string()
    }
    fn uuid() -> Uuid {
        Uuid::parse_str("5105ed71-f116-4fd1-b6df-8dc5f73ee73c").unwrap()
    }
}

// This is a test of this test module, we do not mark it as a test here,
// since otherwise it will be run every time another test includes this module
#[allow(dead_code)]
pub fn test_test_module() {
    let mut test_data = TestDataSection::default();
    let uuid = test_data.logging_uuid;

    let transaction_valid_word = TestTransaction::SetWord("Test".to_string());
    let transaction_invalid_word = TestTransaction::SetWord("Test Test".to_string());
    let transaction_valid_number = TestTransaction::SetNumber(3);
    let transaction_invalid_number = TestTransaction::SetNumber(2);
    let transaction_reset = TestTransaction::ResetIfNumberIsOver100;
    let transaction_large_number = TestTransaction::SetNumber(103);

    // Try applying all 4 transactions
    clear_transaction_log(uuid);
    DataSection::apply(&mut test_data, transaction_valid_word.clone());
    assert_eq!(
        test_data.single_word, "Test",
        "Transaction should have been applied"
    );

    DataSection::apply(&mut test_data, transaction_invalid_word.clone());
    assert_eq!(
        test_data.single_word, "Test",
        "Transaction should have been skipped"
    );

    DataSection::apply(&mut test_data, transaction_valid_number.clone());
    assert_eq!(
        test_data.odd_number, 3,
        "Transaction should have been applied"
    );

    DataSection::apply(&mut test_data, transaction_invalid_number.clone());
    assert_eq!(
        test_data.odd_number, 3,
        "Transaction should have been skipped"
    );

    DataSection::apply(&mut test_data, transaction_reset.clone());
    assert_eq!(
        test_data.odd_number, 3,
        "Transaction should have been skipped"
    );

    DataSection::apply(&mut test_data, transaction_large_number.clone());
    assert_eq!(
        test_data.odd_number, 103,
        "Transaction should have been applied"
    );

    DataSection::apply(&mut test_data, transaction_reset.clone());
    assert_eq!(
        test_data.odd_number, 0,
        "Transaction should have been applied"
    );

    // Test if the transaction log is correct
    let log = get_transaction_log(uuid);
    assert_eq!(
        log,
        vec![
            (TransactionStatus::Applied, transaction_valid_word.clone()),
            (TransactionStatus::Skipped, transaction_invalid_word.clone()),
            (TransactionStatus::Applied, transaction_valid_number.clone()),
            (
                TransactionStatus::Skipped,
                transaction_invalid_number.clone()
            ),
            (TransactionStatus::Skipped, transaction_reset.clone()),
            (TransactionStatus::Applied, transaction_large_number.clone()),
            (TransactionStatus::Applied, transaction_reset.clone()),
        ],
    );
}
