use module::{DocumentTransaction, Module};

// TODO: complete docs

/// A transaction that can be applied to a [`Session`].
///
/// [`Session`]: crate::document::Session
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TransactionArgs<M: Module> {
    Document(<M::DocumentData as DocumentTransaction>::Args),
    User(<M::UserData as DocumentTransaction>::Args),
    Session(<M::SessionData as DocumentTransaction>::Args),
    Shared(<M::SharedData as DocumentTransaction>::Args),
}

/// The output of a transaction applied to a [`Session`].
///
/// [`Session`]: crate::document::Session
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionOutput<M: Module> {
    Document(<M::DocumentData as DocumentTransaction>::Output),
    User(<M::UserData as DocumentTransaction>::Output),
    Session(<M::SessionData as DocumentTransaction>::Output),
    Shared(<M::SharedData as DocumentTransaction>::Output),
}

/// Common error type for all transactions on a document.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionError<M: Module> {
    Document(<M::DocumentData as DocumentTransaction>::Error),
    User(<M::UserData as DocumentTransaction>::Error),
    Session(<M::SessionData as DocumentTransaction>::Error),
    Shared(<M::SharedData as DocumentTransaction>::Error),
}

/// The error that can occur when applying a transaction to a [`Session`].
///
/// [`Session`]: crate::document::Session
#[derive(Debug, Clone, PartialEq)]
pub enum SessionApplyError<M: Module> {
    TransactionFailure(TransactionError<M>),
    MissingProject,
    MissingDocument,
}
