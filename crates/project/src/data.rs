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
    Change, ChangeBuilder, PendingChange, ProjectSource,
};

/// Unique identifier of a data section in a [`crate::Project`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
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

/// A read-only view to a data section in a [`ProjectView`].
///
/// [`DataView`] provides access to the 4 data sections associated with a specific module `M`.
/// It allows you to inspect the persistent, user-specific, session, and shared data, but not to modify the project directly.
/// Modifications must be made through a [`ChangeBuilder`] and applied to the [`crate::Project`] using [`crate::Project::apply_changes`].
#[derive(Clone, Debug)]
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
    /// Unique identifier to associate a project with its views and [`ChangeBuilder`]s
    pub(crate) uuid: Uuid,
}

impl<M: Module> ProjectSource for DataView<'_, M> {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl<M: Module> From<DataView<'_, M>> for DataId {
    fn from(dv: DataView<'_, M>) -> Self {
        dv.id
    }
}

impl<M: Module> DataView<'_, M> {
    /// Plans to apply a transaction to [`Module::PersistentData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_persistent(
        &self,
        args: <M::PersistentData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes.push(PendingChange::Change(Change::Transaction {
            id: self.id,
            args: DataTransactionArgs::<M>(args).into(),
        }));
    }

    /// Plans to apply a transaction to [`Module::PersistentUserData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_persistent_user(
        &self,
        args: <M::PersistentUserData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes
            .push(PendingChange::Change(Change::UserTransaction {
                id: self.id,
                args: UserDataTransactionArgs::<M>(args).into(),
            }));
    }

    /// Plans to apply a transaction to [`Module::SessionData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_session(
        &self,
        args: <M::SessionData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes.push(PendingChange::SessionTransaction {
            id: self.id,
            args: SessionDataTransactionArgs::<M>(args).into(),
        });
    }

    /// Plans to apply a transaction to [`Module::SharedData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_shared(&self, args: <M::SharedData as DataSection>::Args, cb: &mut ChangeBuilder) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes.push(PendingChange::SharedTransaction {
            id: self.id,
            args: SharedDataTransactionArgs::<M>(args).into(),
        });
    }

    /// Plans the deletion of this data
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes
            .push(PendingChange::Change(Change::DeleteData(self.id)));
    }

    /// Plans to move this data section to another document.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `new_owner` - The document to move the data to.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn move_to_document(&self, new_owner: &crate::DocumentView, cb: &mut ChangeBuilder) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes.push(PendingChange::Change(Change::MoveData {
            id: self.id,
            new_owner: Some(new_owner.id),
        }));
    }

    /// Plans to make this data section an orphan (not owned by any document).
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn make_orphan(&self, cb: &mut ChangeBuilder) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        // TODO: what if we call this multiple times?
        cb.changes.push(PendingChange::Change(Change::MoveData {
            id: self.id,
            new_owner: None,
        }));
    }
}

/// Pending version of [`DataView`] that does not yet exist in the [`ProjectView`].
#[derive(Debug)]
pub struct PlannedData<'a, 'b, M: Module> {
    pub id: DataId,
    pub project: &'a ProjectView,
    pub(crate) phantomdata: PhantomData<M>,
    pub(crate) cb: &'b mut ChangeBuilder,
}

impl<M: Module> From<PlannedData<'_, '_, M>> for DataId {
    fn from(dv: PlannedData<'_, '_, M>) -> Self {
        dv.id
    }
}

impl<M: Module> Deref for PlannedData<'_, '_, M> {
    type Target = DataId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
impl<M: Module> PlannedData<'_, '_, M> {
    /// Plans to apply a transaction to [`Module::PersistentData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to the [`ChangeBuilder`]
    /// used to create this [`PlannedData`].
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_persistent(&mut self, args: <M::PersistentData as DataSection>::Args) {
        self.cb
            .changes
            .push(PendingChange::Change(Change::Transaction {
                id: self.id,
                args: DataTransactionArgs::<M>(args).into(),
            }));
    }

    /// Plans to apply a transaction to [`Module::PersistentUserData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to the [`ChangeBuilder`]
    /// used to create this [`PlannedData`].
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_persistent_user(&mut self, args: <M::PersistentUserData as DataSection>::Args) {
        self.cb
            .changes
            .push(PendingChange::Change(Change::UserTransaction {
                id: self.id,
                args: UserDataTransactionArgs::<M>(args).into(),
            }));
    }

    /// Plans to apply a transaction to [`Module::SessionData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to the [`ChangeBuilder`]
    /// used to create this [`PlannedData`].
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_session(&mut self, args: <M::SessionData as DataSection>::Args) {
        self.cb.changes.push(PendingChange::SessionTransaction {
            id: self.id,
            args: SessionDataTransactionArgs::<M>(args).into(),
        });
    }

    /// Plans to apply a transaction to [`Module::SharedData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to the [`ChangeBuilder`]
    /// used to create this [`PlannedData`].
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    pub fn apply_shared(&mut self, args: <M::SharedData as DataSection>::Args) {
        self.cb.changes.push(PendingChange::SharedTransaction {
            id: self.id,
            args: SharedDataTransactionArgs::<M>(args).into(),
        });
    }
}
