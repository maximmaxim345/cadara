pub mod internal;

use crate::ProjectView;

use super::{
    internal::{AppliedTransaction, InternalData, TransactionState, UndoUnit, UndoneTransaction},
    transaction,
};
use internal::InternalDataSession;
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
    // TODO: add doc
    fn apply_session(
        &self,
        args: <M::SessionData as DataTransaction>::Args,
    ) -> Result<<M::SessionData as DataTransaction>::Output, transaction::SessionApplyError<M>>
    {
        let mut internal = self.session.lock().unwrap();
        // We do not need to replicate the transaction to other sessions.
        internal.session_data.apply(args).map_or_else(
            |error| {
                Result::Err(transaction::SessionApplyError::TransactionFailure(
                    transaction::TransactionError::Session(error),
                ))
            },
            Result::Ok,
        )
    }

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
    pub fn apply(
        &mut self,
        args: transaction::TransactionArgs<M>,
    ) -> Result<transaction::TransactionOutput<M>, transaction::SessionApplyError<M>> {
        use transaction::TransactionArgs as Args;

        if let Args::Session(session_args) = args {
            // Session data is not synced with other sessions, so we can just directly apply it
            self.apply_session(session_args)
                .map_or_else(Result::Err, |output| {
                    Ok(transaction::TransactionOutput::Session(output))
                })
        } else {
            // The remaining transaction are applied through the data model
            // This is because they need to be synced with other session.
            let session_uuid = self.session.lock().unwrap().session_uuid;
            let ref_cell = &self.data_model_ref.upgrade().unwrap();
            let mut internal_data = ref_cell.lock().unwrap();
            match args {
                Args::Persistent(data_args) => internal_data
                    .apply_persistent(data_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::Persistent(output))
                    }),
                Args::PersistentUser(user_args) => internal_data
                    .apply_user(user_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::PersistentUser(output))
                    }),
                Args::Shared(shared_args) => internal_data
                    .apply_shared(&shared_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::Shared(output))
                    }),
                // We already handled this case above
                Args::Session(_) => unreachable!(),
            }
        }
    }
}
