use module::{DataTransaction, Module};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{module_data::TransactionData, project::ProjectView, Change, ChangeBuilder};

/// Unique identifier of a data section in a [`Project`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct DataUuid {
    uuid: Uuid,
}

impl DataUuid {
    pub(crate) const fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    #[must_use]
    pub fn new_v4() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// A read only view to a data section in a [`ProjectView`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct DataView<'a, M: Module> {
    pub project: &'a ProjectView,
    pub data: DataUuid,
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
        cb.changes.push(Change::Transaction {
            uuid: self.data,
            data: TransactionData::<M>(args).into(),
        });
    }
}
