use crate::{data::transaction::TransactionError, user::User};
use module::{DataTransaction, Module, ReversibleDataTransaction};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, Weak},
};
use uuid::Uuid;

use super::transaction::SessionApplyError;

// TODO: write docs for these types
/// Data required to undo and redo a transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoUnit<T: ReversibleDataTransaction> {
    pub undo_data: T::UndoData,
    pub args: <T as DataTransaction>::Args,
}

/// Data required to redo a transaction.
type RedoUnit<T> = <T as DataTransaction>::Args;

// TODO: rename these types
/// Represents the state of a document transaction that can be reversed.
#[derive(Debug, Clone, PartialEq)]
pub enum AppliedTransaction<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
    Persistent(UndoUnit<D>),
    PersistentUser(UndoUnit<U>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UndoneTransaction<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
    Persistent(RedoUnit<D>),
    PersistentUser(RedoUnit<U>),
}

/// Represents the state of a transaction that can be reversed.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
    Applied(AppliedTransaction<D, U>),
    Undone(UndoneTransaction<D, U>),
    Failed(UndoneTransaction<D, U>),
}

/// Represents the state of a transaction history.
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionHistoryState<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
    pub session: Uuid,
    pub name: String,
    pub state: TransactionState<D, U>,
}
