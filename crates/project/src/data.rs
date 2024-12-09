//! Data and state management.
//!
//! Each data section is described by a [`module::Module`]. Each data section is a member of a single document and
//! has a project wide unique Uuid, which is stable across renames.

// Public modules and re-exports
pub mod transaction;
use serde::{Deserialize, Serialize};
pub use session::{DataView, Snapshot};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
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

// Internal modules
pub(crate) mod internal;
pub(crate) mod session;
