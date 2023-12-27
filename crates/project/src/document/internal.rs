use crate::{
    document::transaction::TransactionError,
    transaction::{DocumentTransaction, ReversibleDocumentTransaction},
    user::User,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Weak,
};
use uuid::Uuid;

use super::{session::internal::InternalDocumentSession, transaction::SessionApplyError, Module};

// TODO: write docs for these types
/// Data required to undo and redo a transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoUnit<T: ReversibleDocumentTransaction> {
    pub undo_data: T::UndoData,
    pub args: <T as DocumentTransaction>::Args,
}

/// Data required to redo a transaction.
type RedoUnit<T> = <T as DocumentTransaction>::Args;

// TODO: rename these types
/// Represents the state of a document transaction that can be reversed.
#[derive(Debug, Clone, PartialEq)]
pub enum AppliedTransaction<D: ReversibleDocumentTransaction, U: ReversibleDocumentTransaction> {
    Document(UndoUnit<D>),
    User(UndoUnit<U>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UndoneTransaction<D: ReversibleDocumentTransaction, U: ReversibleDocumentTransaction> {
    Document(RedoUnit<D>),
    User(RedoUnit<U>),
}

/// Represents the state of a transaction that can be reversed.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState<D: ReversibleDocumentTransaction, U: ReversibleDocumentTransaction> {
    Applied(AppliedTransaction<D, U>),
    Undone(UndoneTransaction<D, U>),
    Failed(UndoneTransaction<D, U>),
}

/// Represents the state of a transaction history.
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionHistoryState<
    D: ReversibleDocumentTransaction,
    U: ReversibleDocumentTransaction,
> {
    pub session: Uuid,
    pub name: String,
    pub state: TransactionState<D, U>,
}

// TODO: rename to InternalDcoument
// TODO: make this more private
/// Represents an internal model of a document within a project in `CADara`.
///
/// Used internally by [`Project`] to store data about a document.
///
/// [`Project`]: crate::Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalDocumentModel<M: Module> {
    /// Document data for this document
    pub(crate) document_data: M::DocumentData,
    /// TODO: write doc
    #[serde(skip)]
    pub transaction_history: VecDeque<TransactionHistoryState<M::DocumentData, M::UserData>>,
    /// User-specific data for this document
    pub(crate) user_data: M::UserData,
    /// Shared session data for this document
    // TODO: this was an option
    // TODO: make this a skip conditional (sometimes we might want to deserialize this too)
    #[serde(skip)]
    pub(crate) shared_data: Option<M::SharedData>,
    /// List of all currently open sessions of this document.
    #[serde(skip)]
    pub sessions: Vec<(Uuid, Weak<RefCell<InternalDocumentSession<M>>>)>,
    /// UUID of the module implementing the document.
    // TODO: remove duplicate in serialization
    pub module_uuid: Uuid,
    // TODO: write doc
    #[serde(skip)]
    pub(crate) session_to_user: HashMap<Uuid, User>,
}

// TODO: make methods private, write docs
impl<M: Module> InternalDocumentModel<M> {
    pub fn apply_document(
        &mut self,
        args: <M::DocumentData as DocumentTransaction>::Args,
        session_uuid: Uuid,
    ) -> Result<<M::DocumentData as DocumentTransaction>::Output, SessionApplyError<M>> {
        // First we try to apply the transaction to our internal data
        let (output, undo_data) =
            ReversibleDocumentTransaction::apply(&mut self.document_data, args.clone()).map_err(
                |e| SessionApplyError::TransactionFailure(TransactionError::<M>::Document(e)),
            )?;

        let name = <M::DocumentData as DocumentTransaction>::undo_history_name(&args);

        // We can now apply the transaction to all sessions
        for session in &self.sessions {
            let session = session.1.upgrade().unwrap();
            ReversibleDocumentTransaction::apply_unchecked(
                &mut session.borrow_mut().document_data,
                args.clone(),
            );
        }

        // Now we need to store the undo data and args for later undoing
        self.transaction_history.push_back(TransactionHistoryState {
            session: session_uuid,
            name,
            state: TransactionState::Applied(AppliedTransaction::Document(UndoUnit {
                undo_data,
                args,
            })),
        });

        // And return the output
        Ok(output)
    }

    pub fn apply_user(
        &mut self,
        args: <M::UserData as DocumentTransaction>::Args,
        session_uuid: Uuid,
    ) -> Result<<M::UserData as DocumentTransaction>::Output, SessionApplyError<M>> {
        // For now we just do the same thing as apply_document, since we haven't implemented
        // multiple users yet

        // for now we assume that there is only one user
        // TODO: implement multiple users
        let _user_uuid = *self.session_to_user.get(&session_uuid).unwrap();

        // First we try to apply the transaction to our internal data
        let (output, undo_data) =
            ReversibleDocumentTransaction::apply(&mut self.user_data, args.clone()).map_err(
                |e| SessionApplyError::TransactionFailure(TransactionError::<M>::User(e)),
            )?;
        let name = <M::UserData as DocumentTransaction>::undo_history_name(&args);

        // We can now apply the transaction to all sessions
        for session in &self.sessions {
            let session = session.1.upgrade().unwrap();
            ReversibleDocumentTransaction::apply_unchecked(
                &mut session.borrow_mut().user_data,
                args.clone(),
            );
        }

        // Now we need to store the undo data and args for later undoing
        // TODO: explain why we have a central list for all sessions
        self.transaction_history.push_back(TransactionHistoryState {
            session: session_uuid,
            name,
            state: TransactionState::Applied(AppliedTransaction::User(UndoUnit {
                undo_data,
                args,
            })),
        });

        // And return the output
        Ok(output)
    }

    pub fn apply_shared(
        &mut self,
        args: &<M::SharedData as DocumentTransaction>::Args,
        _session_uuid: Uuid,
    ) -> Result<<M::SharedData as DocumentTransaction>::Output, SessionApplyError<M>> {
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
