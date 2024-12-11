use module::{DataTransaction, Module};

// TODO: complete docs

/// A transaction that can be applied to a [`DataSession`].
///
/// [`DataSession`]: crate::data::DataSession
#[derive(Debug, Clone, PartialEq, Hash)]
#[expect(clippy::module_name_repetitions)]
pub enum TransactionArgs<M: Module> {
    Persistent(<M::PersistentData as DataTransaction>::Args),
    PersistentUser(<M::PersistentUserData as DataTransaction>::Args),
    Session(<M::SessionData as DataTransaction>::Args),
    Shared(<M::SharedData as DataTransaction>::Args),
}

/// The output of a transaction applied to a [`DataSession`].
///
/// [`DataSession`]: crate::data::DataSession
#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::module_name_repetitions)]
pub enum TransactionOutput<M: Module> {
    Persistent(<M::PersistentData as DataTransaction>::Output),
    PersistentUser(<M::PersistentUserData as DataTransaction>::Output),
    Session(<M::SessionData as DataTransaction>::Output),
    Shared(<M::SharedData as DataTransaction>::Output),
}

/// Common error type for all transactions on a document.
#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::module_name_repetitions)]
pub enum TransactionError<M: Module> {
    Persistent(<M::PersistentData as DataTransaction>::Error),
    PersistentUser(<M::PersistentUserData as DataTransaction>::Error),
    Session(<M::SessionData as DataTransaction>::Error),
    Shared(<M::SharedData as DataTransaction>::Error),
}

/// The error that can occur when applying a transaction to a [`DataSession`].
///
/// [`DataSession`]: crate::data::DataSession
#[derive(Debug, Clone, PartialEq)]
pub enum SessionApplyError<M: Module> {
    TransactionFailure(TransactionError<M>),
    MissingProject,
    MissingDocument,
}
