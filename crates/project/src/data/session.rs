use crate::{
    Change, ChangeBuilder, ErasedTransactionData, ProjectView, TransactionData, TransactionTarget,
};

use super::{transaction, DataUuid};
use module::{DataTransaction, Module, ReversibleDataTransaction};
use std::sync::{Arc, Mutex, Weak};

/// Represents an interactive session of a document within a project.
///
/// A [`DataSession`] encapsulates the state of an open document that is part of a [`Project`].
/// It maintains a copy of the document's state, allowing for concurrent editing and individual
/// management of persistent and non-persistent data.
///
/// Modifications to the document are made by passing [`transaction::TransactionArgs`] through [`DataSession::apply`].
///
/// [`Project`]: crate::Project
#[derive(Clone, Debug)]
pub struct DataView<'a, M: Module> {
    pub project: &'a ProjectView,
    pub data: DataUuid,
    /// Persistent data for this session.
    ///
    /// Synced with other sessions and the project.
    pub persistent: &'a M::PersistentData,
    /// Persistent user-specific data for this session.
    pub persistent_user: &'a M::PersistentUserData,
    /// Non-persistent user-specific data for this session.
    pub session_data: &'a M::SessionData,
    /// Non-persistent data shared among users for this session.
    pub shared_data: &'a M::SharedData,
}

impl<'a, M: Module> DataView<'a, M> {
    /// Applies a transaction to this data session.
    ///
    /// This function handles different types of transactions and applies them to the appropriate
    /// data section. It ensures that transactions are properly synchronized across sessions when necessary.
    ///
    /// # Arguments
    ///
    /// * `args` - A `TransactionArgs<M>` representing the transaction to be applied.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the output of the successfully applied transaction.
    ///
    /// # Errors
    ///
    /// Returns a `SessionApplyError<M>` if the transaction fails to apply to the requested data section.
    ///
    /// # Behavior
    ///
    /// - For `Session` transactions: Applied directly to the current session without synchronization.
    /// - For `Persistent`, `PersistentUser`, and `Shared` transactions: Applied through the data model
    ///   to ensure synchronization across all sessions.
    pub fn apply(&mut self, args: transaction::TransactionArgs<M>, cb: &mut ChangeBuilder) {
        match args {
            transaction::TransactionArgs::Persistent(p) => {
                cb.changes.push(Change::Transaction(ErasedTransactionData {
                    uuid: M::uuid(),
                    target: TransactionTarget::PersistentData(self.data),
                    data: Box::new(TransactionData::<M>(p)),
                }));
            }
            transaction::TransactionArgs::PersistentUser(_) => todo!(),
            transaction::TransactionArgs::Session(_) => todo!(),
            transaction::TransactionArgs::Shared(_) => todo!(),
        }
    }
}
