//! On-disk log entry shape for the CRDT layer.
//!
//! Each [`LogEntry`] carries a totally-ordered `(lamport, session)` key.
//! Replicas that have seen the same set of entries produce identical
//! `Vec<LogEntry>` after sort by `(lamport, session)`.

use crate::branch::BranchId;
use crate::checkpoint::CheckpointId;
use crate::module_data::{
    ErasedDataTransactionArgs, ErasedSessionDataTransactionArgs, ErasedSharedDataTransactionArgs,
    ErasedUserDataTransactionArgs, ModuleId,
};
use crate::user::{SessionId, UserId};
use crate::{DataId, DocumentId, FolderPath, FolderTarget, Path};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Single entry in [`Project::log`](crate::Project::log).
///
/// Replicas converge by sorting their union by `(lamport, session)`.
/// `wall_clock` is display-only and never read by replay; see the CRDT design spec.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub(crate) lamport: u64,
    pub(crate) session: SessionId,
    #[serde(default)]
    pub(crate) wall_clock: Option<SystemTime>,
    pub(crate) payload: LogPayload,
}

/// What a [`LogEntry`] carries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogPayload {
    /// Atomic group of changes recorded by one [`ChangeBuilder`](crate::ChangeBuilder)
    /// application. Undoable as one unit.
    Changes(Vec<Change>),
    /// Cancel the most recent live `Changes` or `MergeBranch` from this session.
    Undo,
    /// Reinstate the most recently cancelled entry from this session (LIFO).
    Redo,
    /// Declare a new session belonging to `user` on `branch`. Must appear in
    /// the log before any other entry referring to its `SessionId`.
    NewSession { user: UserId, branch: BranchId },
    /// Named position in the log. Display/audit only.
    Checkpoint(CheckpointId),
    /// Declare a one-shot snapshot import of `from` into `into`. Undoable.
    MergeBranch { from: BranchId, into: BranchId },
}

/// One change to the persistent state of a [`Project`](crate::Project).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Change {
    CreateDocument {
        id: DocumentId,
        path: Path,
    },
    DeleteDocument(DocumentId),
    RenameDocument {
        id: DocumentId,
        new_name: String,
    },
    MoveDocument {
        id: DocumentId,
        new_folder: FolderPath,
    },
    MoveFolder {
        old_path: Path,
        new_path: FolderTarget,
    },
    CreateData {
        id: DataId,
        module: ModuleId,
        owner: Option<DocumentId>,
    },
    DeleteData(DataId),
    MoveData {
        id: DataId,
        new_owner: Option<DocumentId>,
    },
    Transaction {
        id: DataId,
        args: ErasedDataTransactionArgs,
    },
    UserTransaction {
        id: DataId,
        args: ErasedUserDataTransactionArgs,
    },
}

/// Non-persistent pending change recorded in a [`ChangeBuilder`](crate::ChangeBuilder).
///
/// `SessionTransaction` and `SharedTransaction` are applied to in-memory
/// shared/session state at `apply_changes` time without going into the log.
// Only used within this crate; pub(crate) is the right visibility but
// clippy's redundant_pub_crate fires inside a private module.
#[allow(clippy::redundant_pub_crate)]
#[derive(Clone, Debug)]
pub(crate) enum PendingChange {
    Change(Change),
    Undo,
    Redo,
    MergeBranch { from: BranchId, into: BranchId },
    SessionTransaction {
        id: DataId,
        args: ErasedSessionDataTransactionArgs,
    },
    SharedTransaction {
        id: DataId,
        args: ErasedSharedDataTransactionArgs,
    },
}
