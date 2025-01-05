//! Branching and merging.
//!
//! Branches in [`Project`](crate::Project)s are used to indicate (past) branching of a [`Project`](crate::Project).
//! While leave nodes internally exist as separate [`Project`](crate::Project)s, merging them combines
//! the history to a single linear [`Project::log`](crate::Project::log).
//!
//! Giving branches a unique [`BranchId`] allows us to reconstruct the non-linear history from
//! a single linear log.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier of a branch in a [`crate::Project`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct BranchId(Uuid);

impl BranchId {
    /// Creates a new random branch identifier.
    #[must_use]
    #[expect(dead_code)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a new identifier for the main branch.
    ///
    /// The `main` branch has a special constant [`BranchId`].
    #[must_use]
    pub const fn main() -> Self {
        Self(Uuid::from_u128(0))
    }
}

impl Default for BranchId {
    fn default() -> Self {
        Self::main()
    }
}
