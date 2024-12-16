use module::{DataSection, Module};
use serde::{Deserialize, Serialize};
use std::{fmt, marker::PhantomData, ops::Deref};
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

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataId({})", self.0)
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

impl<M: Module> From<DataView<'_, M>> for DataId {
    fn from(dv: DataView<'_, M>) -> Self {
        dv.id
    }
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
        &self,
        args: <M::PersistentData as DataSection>::Args,
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
        &self,
        args: <M::PersistentUserData as DataSection>::Args,
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
        &self,
        args: <M::SessionData as DataSection>::Args,
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
    pub fn apply_shared(&self, args: <M::SharedData as DataSection>::Args, cb: &mut ChangeBuilder) {
        cb.changes.push(PendingChange::SharedTransaction {
            id: self.id,
            args: SharedDataTransactionArgs::<M>(args).into(),
        });
    }

    /// Plans the deletion of this data
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        cb.changes
            .push(PendingChange::Change(Change::DeleteData(self.id)));
    }

    /// Plans to move this data section to another document.
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `new_owner` - The document to move the data to.
    pub fn move_to_document(&self, new_owner: &crate::DocumentView, cb: &mut ChangeBuilder) {
        cb.changes.push(PendingChange::Change(Change::MoveData {
            id: self.id,
            new_owner: Some(new_owner.id),
        }));
    }

    /// Plans to make this data section an orphan (not owned by any document).
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    pub fn make_orphan(&self, cb: &mut ChangeBuilder) {
        // TODO: what if we call this multiple times?
        cb.changes.push(PendingChange::Change(Change::MoveData {
            id: self.id,
            new_owner: None,
        }));
    }
}

/// Pending version of [`DataView`] that does not yet exist in the [`ProjectView`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct PlannedData<'a, M: Module> {
    pub id: DataId,
    pub project: &'a ProjectView,
    pub phantomdata: PhantomData<M>,
}

impl<M: Module> From<PlannedData<'_, M>> for DataId {
    fn from(dv: PlannedData<'_, M>) -> Self {
        dv.id
    }
}

impl<M: Module> Deref for PlannedData<'_, M> {
    type Target = DataId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
impl<M: Module> PlannedData<'_, M> {
    /// Plans to apply a transaction to [`Module::PersistentData`].
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_persistent(
        &self,
        args: <M::PersistentData as DataSection>::Args,
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
        &self,
        args: <M::PersistentUserData as DataSection>::Args,
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
        &self,
        args: <M::SessionData as DataSection>::Args,
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
    pub fn apply_shared(&self, args: <M::SharedData as DataSection>::Args, cb: &mut ChangeBuilder) {
        cb.changes.push(PendingChange::SharedTransaction {
            id: self.id,
            args: SharedDataTransactionArgs::<M>(args).into(),
        });
    }
}
