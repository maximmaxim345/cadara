use module::{DataSection, Module, ReversibleDataTransaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    ApplyFailed,
    Applied,
    Undone,
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

#[derive(Clone, Hash, PartialEq, Debug)]
pub enum TestTransaction {
    SetWord(String),
    SetNumber(i32),
    FailIfNumberIsOver100,
}

#[derive(Clone, Hash, PartialEq, Debug)]
pub enum TestTransactionUndoData {
    SetWord { before: String, after: String },
    SetNumber { before: i32, after: i32 },
}

#[derive(Clone, PartialEq, Debug)]
pub enum TestTransactionError {
    InvalidString,
    InvalidNumber,
}

impl DataSection for TestDataSection {
    type Args = TestTransaction;
    type Error = TestTransactionError;
    type Output = String;
    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use the undoable transaction to implement this
        <Self as ReversibleDataTransaction>::apply(self, args).map(|(output, _undo_data)| output)
    }

    fn undo_history_name(args: &Self::Args) -> String {
        match args {
            TestTransaction::SetWord(word) => format!("Set word to {word}"),
            TestTransaction::SetNumber(number) => format!("Set number to {number}"),
            TestTransaction::FailIfNumberIsOver100 => "Fail if number is over 100".to_string(),
        }
    }
}

impl ReversibleDataTransaction for TestDataSection {
    type UndoData = (TestTransactionUndoData, Self::Args);
    fn apply(&mut self, args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error> {
        let result = match args.clone() {
            TestTransaction::SetWord(word) => {
                if word.contains(' ') {
                    Result::Err(TestTransactionError::InvalidString)
                } else {
                    let old_word = self.single_word.clone();
                    let message = format!("changed word from {old_word} to {word}");
                    self.single_word.clone_from(&word);
                    Result::Ok((
                        message,
                        TestTransactionUndoData::SetWord {
                            before: old_word,
                            after: word,
                        },
                    ))
                }
            }
            TestTransaction::SetNumber(number) => {
                if number % 2 == 0 {
                    Result::Err(TestTransactionError::InvalidNumber)
                } else {
                    let old_number = self.odd_number;
                    self.odd_number = number;
                    let message = format!("changed number from {old_number} to {number}");
                    Result::Ok((
                        message,
                        TestTransactionUndoData::SetNumber {
                            before: old_number,
                            after: number,
                        },
                    ))
                }
            }
            TestTransaction::FailIfNumberIsOver100 => {
                if self.odd_number > 100 {
                    Result::Err(TestTransactionError::InvalidNumber)
                } else {
                    Result::Ok((
                        "".to_string(),
                        TestTransactionUndoData::SetNumber {
                            before: self.odd_number,
                            after: self.odd_number,
                        },
                    ))
                }
            }
        };
        let mut map = GLOBAL_TRANSACTION_LOG.write().unwrap();
        let log = map.entry(self.logging_uuid).or_default();

        // log the result
        match result {
            Err(err) => {
                log.push((TransactionStatus::ApplyFailed, args.clone()));
                Err(err)
            }
            Ok((output, undo_data)) => {
                log.push((TransactionStatus::Applied, args.clone()));
                Ok((output, (undo_data, args)))
            }
        }
    }
    fn undo(&mut self, undo_data: Self::UndoData) {
        let mut map = GLOBAL_TRANSACTION_LOG.write().unwrap();
        let log = map.entry(self.logging_uuid).or_default();

        // log the undo operation
        log.push((TransactionStatus::Undone, undo_data.1));
        match undo_data.0 {
            TestTransactionUndoData::SetWord { before, after } => {
                assert_eq!(self.single_word, after);
                self.single_word = before;
            }
            TestTransactionUndoData::SetNumber { before, after } => {
                assert_eq!(self.odd_number, after);
                self.odd_number = before;
            }
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
    let test_data_before = test_data.clone();
    let uuid = test_data.logging_uuid;

    let transaction_valid_word = TestTransaction::SetWord("Test".to_string());
    let transaction_invalid_word = TestTransaction::SetWord("Test Test".to_string());
    let transaction_valid_number = TestTransaction::SetNumber(3);
    let transaction_invalid_number = TestTransaction::SetNumber(2);

    // Try applying all 4 transactions
    clear_transaction_log(uuid);
    let undodata_word =
        ReversibleDataTransaction::apply(&mut test_data, transaction_valid_word.clone())
            .unwrap()
            .1;
    assert_eq!(
        test_data.single_word, "Test",
        "Transaction should have been applied"
    );

    let result = DataSection::apply(&mut test_data, transaction_invalid_word.clone());
    assert!(result.is_err());

    let undodata_number =
        ReversibleDataTransaction::apply(&mut test_data, transaction_valid_number.clone())
            .unwrap()
            .1;
    assert_eq!(
        test_data.odd_number, 3,
        "Transaction should have been applied"
    );

    let result = DataSection::apply(&mut test_data, transaction_invalid_number.clone());
    assert!(result.is_err());

    // Test if the transaction log is correct
    let log = get_transaction_log(uuid);
    assert_eq!(
        log,
        vec![
            (TransactionStatus::Applied, transaction_valid_word.clone()),
            (
                TransactionStatus::ApplyFailed,
                transaction_invalid_word.clone()
            ),
            (TransactionStatus::Applied, transaction_valid_number.clone()),
            (
                TransactionStatus::ApplyFailed,
                transaction_invalid_number.clone()
            ),
        ],
    );

    // Try undoing the 2 successful transactions
    clear_transaction_log(uuid);
    ReversibleDataTransaction::undo(&mut test_data, undodata_number);
    ReversibleDataTransaction::undo(&mut test_data, undodata_word);
    let log = get_transaction_log(uuid);
    assert_eq!(
        log,
        vec![
            (TransactionStatus::Undone, transaction_valid_number.clone()),
            (TransactionStatus::Undone, transaction_valid_word.clone()),
        ],
    );

    assert_eq!(
        test_data, test_data_before,
        "All Transactions should have been undone"
    );

    // Try the failing transaction
    assert!(DataSection::apply(&mut test_data, TestTransaction::SetNumber(99)).is_ok());
    assert!(DataSection::apply(&mut test_data, TestTransaction::FailIfNumberIsOver100).is_ok());
    assert!(DataSection::apply(&mut test_data, TestTransaction::SetNumber(101)).is_ok());
    assert!(DataSection::apply(&mut test_data, TestTransaction::FailIfNumberIsOver100).is_err());
}
