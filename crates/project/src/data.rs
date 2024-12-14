use module::{DataTransaction, Module};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    module_data::{
        DataTransactionArgs, SessionDataTransactionArgs, SharedDataTransactionArgs,
        UserDataTransactionArgs,
    },
    project::ProjectView,
    Change, ChangeBuilder, PendingChange,
};

/// Unique identifier of a data section in a [`Project`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct DataId(Uuid);

impl DataId {
    /// Create a new random identifier.
    #[must_use]
    pub(crate) fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A read only view to a data section in a [`ProjectView`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct DataView<'a, M: Module> {
    pub project: &'a ProjectView,
    pub id: DataId,
    /// Persistent data shared by all users.
    pub persistent: &'a M::PersistentData,
    /// Persistent user-specific data.
    pub persistent_user: &'a M::PersistentUserData,
    /// Non-persistent user-specific data.
    pub session_data: &'a M::SessionData,
    /// Non-persistent data also shared among other users.
    pub shared_data: &'a M::SharedData,
}

impl<M: Module> DataView<'_, M> {
    /// Plans to apply a transaction to [`Module::PersistentData`].
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_persistent(
        &mut self,
        args: <M::PersistentData as DataTransaction>::Args,
        cb: &mut ChangeBuilder,
    ) {
        cb.changes.push(PendingChange::Change(Change::Transaction {
            id: self.id,
            args: DataTransactionArgs::<M>(args).into(),
        }));
    }

    /// Plans to apply a transaction to [`Module::PersistentUserData`].
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_persistent_user(
        &mut self,
        args: <M::PersistentUserData as DataTransaction>::Args,
        cb: &mut ChangeBuilder,
    ) {
        cb.changes
            .push(PendingChange::Change(Change::UserTransaction {
                id: self.id,
                args: UserDataTransactionArgs::<M>(args).into(),
            }));
    }

    /// Plans to apply a transaction to [`Module::SessionData`].
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_session(
        &mut self,
        args: <M::SessionData as DataTransaction>::Args,
        cb: &mut ChangeBuilder,
    ) {
        cb.changes.push(PendingChange::SessionTransaction {
            id: self.id,
            args: SessionDataTransactionArgs::<M>(args).into(),
        });
    }

    /// Plans to apply a transaction to [`Module::SharedData`].
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_shared(
        &mut self,
        args: <M::SharedData as DataTransaction>::Args,
        cb: &mut ChangeBuilder,
    ) {
        cb.changes.push(PendingChange::SharedTransaction {
            id: self.id,
            args: SharedDataTransactionArgs::<M>(args).into(),
        });
    }
}
