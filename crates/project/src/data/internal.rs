use crate::{data::transaction::TransactionError, user::User};
use module::{DataTransaction, Module, ReversibleDataTransaction};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Weak,
};
use uuid::Uuid;

use super::{session::internal::InternalDataSession, transaction::SessionApplyError};

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

// TODO: make this more private
/// Represents an internal model of a data section within a project in `CADara`.
///
/// Used internally by [`Project`] to store data about a data section.
///
/// [`Project`]: crate::Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalData<M: Module> {
    /// Persistent data
    pub(crate) persistent_data: M::PersistentData,
    /// TODO: write doc
    #[serde(skip)]
    pub transaction_history:
        VecDeque<TransactionHistoryState<M::PersistentData, M::PersistentUserData>>,
    /// User-specific data for this document
    pub(crate) user_data: M::PersistentUserData,
    /// Shared session data for this document
    // TODO: this was an option
    // TODO: make this a skip conditional (sometimes we might want to deserialize this too)
    #[serde(skip)]
    pub(crate) shared_data: Option<M::SharedData>,
    /// List of all currently open sessions of this document.
    #[serde(skip)]
    pub sessions: Vec<(Uuid, Weak<RefCell<InternalDataSession<M>>>)>,
    /// UUID of the module implementing the document.
    // TODO: remove duplicate in serialization
    pub module_uuid: Uuid,
    // TODO: write doc
    #[serde(skip)]
    pub(crate) session_to_user: HashMap<Uuid, User>,
}

// TODO: make methods private, write docs
impl<M: Module> InternalData<M> {
    pub fn apply_persistent(
        &mut self,
        args: <M::PersistentData as DataTransaction>::Args,
        session_uuid: Uuid,
    ) -> Result<<M::PersistentData as DataTransaction>::Output, SessionApplyError<M>> {
        // First we try to apply the transaction to our internal data
        let (output, undo_data) =
            ReversibleDataTransaction::apply(&mut self.persistent_data, args.clone()).map_err(
                |e| SessionApplyError::TransactionFailure(TransactionError::<M>::Persistent(e)),
            )?;

        let name = <M::PersistentData as DataTransaction>::undo_history_name(&args);

        // We can now apply the transaction to all sessions
        for session in &self.sessions {
            let session = session.1.upgrade().unwrap();
            ReversibleDataTransaction::apply_unchecked(
                &mut session.borrow_mut().persistent,
                args.clone(),
            );
        }

        // Now we need to store the undo data and args for later undoing
        self.transaction_history.push_back(TransactionHistoryState {
            session: session_uuid,
            name,
            state: TransactionState::Applied(AppliedTransaction::Persistent(UndoUnit {
                undo_data,
                args,
            })),
        });

        // And return the output
        Ok(output)
    }

    pub fn apply_user(
        &mut self,
        args: <M::PersistentUserData as DataTransaction>::Args,
        session_uuid: Uuid,
    ) -> Result<<M::PersistentUserData as DataTransaction>::Output, SessionApplyError<M>> {
        // For now we just do the same thing as apply_persistent, since we haven't implemented
        // multiple users yet

        // for now we assume that there is only one user
        // TODO: implement multiple users
        let _user_uuid = *self.session_to_user.get(&session_uuid).unwrap();

        // First we try to apply the transaction to our internal data
        let (output, undo_data) =
            ReversibleDataTransaction::apply(&mut self.user_data, args.clone()).map_err(|e| {
                SessionApplyError::TransactionFailure(TransactionError::<M>::PersistentUser(e))
            })?;
        let name = <M::PersistentUserData as DataTransaction>::undo_history_name(&args);

        // We can now apply the transaction to all sessions
        for session in &self.sessions {
            let session = session.1.upgrade().unwrap();
            ReversibleDataTransaction::apply_unchecked(
                &mut session.borrow_mut().persistent_user,
                args.clone(),
            );
        }

        // Now we need to store the undo data and args for later undoing
        // TODO: explain why we have a central list for all sessions
        self.transaction_history.push_back(TransactionHistoryState {
            session: session_uuid,
            name,
            state: TransactionState::Applied(AppliedTransaction::PersistentUser(UndoUnit {
                undo_data,
                args,
            })),
        });

        // And return the output
        Ok(output)
    }

    pub fn apply_shared(
        &mut self,
        args: &<M::SharedData as DataTransaction>::Args,
        _session_uuid: Uuid,
    ) -> Result<<M::SharedData as DataTransaction>::Output, SessionApplyError<M>> {
        // Works like apply_user, but with no distinction between different users

        // TODO: we currently take a session_uuid, think where it is appropriate
        // First we try to apply the transaction to our internal data
        // TODO: remove the unwrap
        let output = self
            .shared_data
            .as_mut()
            .unwrap()
            .apply(args.clone())
            .map_err(|e| SessionApplyError::TransactionFailure(TransactionError::<M>::Shared(e)))?;

        // We can now apply the transaction to all sessions
        for session in &self.sessions {
            let session = session.1.upgrade().unwrap();
            session
                .borrow_mut()
                .shared_data
                .apply_unchecked(args.clone());
        }

        // since this data section does not support undo, we can just return the output
        Ok(output)
    }
}
