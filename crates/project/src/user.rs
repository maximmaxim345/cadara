use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier of a user within `CADara`.
///
/// [`UserId`] serve two main functions:
/// 1. Dictate what [`module::Module::PersistentUserData`] should be used.
/// 2. To create [`SessionId`]s to associate a changes to a project with a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct UserId(Uuid);

impl UserId {
    /// Creates a new user with a randomly generated id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a user reserved for local host operations.
    ///
    /// It is intended for locally saved projects.
    #[must_use]
    pub const fn local() -> Self {
        Self(Uuid::from_u128(0))
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::local()
    }
}

/// Unique identifier of a branch in a project.
///
/// Branches in [`Project`]s are used to indicate (past) branching of a [`Project`].
/// While leave nodes internally exist as separate [`Project`]s, mearging them combines
/// the history to a single linear [`Project::log`].
///
/// Giving branches a unique [`BranchId`] allows us reconstruct the non linear history from
/// a single linear log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
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

/// Unique identifier to associate changes with the origin.
///
/// We don't directly associate project changes to a [`UserId`], but to a
/// [`SessionId`] registered to it through [`crate::ProjectLogEntry::NewSession`].
///
/// This has two main advantages:
/// - undo/redo will be limited to a single session, meaning: A single user can have
///   multiple simultaneous sessions with separate undo/redo history.
/// - merging multiple branches of the same user will not have spooky effecty due to
///   incorrectly associated undo/redo commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random unique identifier of a Session.
    ///
    /// Before use, this must first be registered in [`crate::ProjectLogEntry::NewSession`].
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
