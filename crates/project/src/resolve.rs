//! Pure resolvers for Layers 1 and 2 of the CRDT design.

use crate::branch::BranchId;
use crate::oplog::{LogEntry, LogPayload};
use crate::user::SessionId;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Walk a session's entries (already lamport-sorted) and return the indices
/// (into `entries`) whose effect is currently active.
///
/// Only `Changes` and `MergeBranch` payloads occupy the active/redo
/// bookkeeping. `Undo` and `Redo` mutate that bookkeeping. Other payloads
/// (`NewSession`, `Checkpoint`) are ignored at this layer.
#[allow(dead_code, reason = "consumed by create_view in the next task")]
pub fn per_session_active_set(entries: &[&LogEntry]) -> BTreeSet<usize> {
    let mut active: BTreeSet<usize> = BTreeSet::new();
    let mut redo_buf: Vec<usize> = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        match entry.payload {
            LogPayload::Changes(_) | LogPayload::MergeBranch { .. } => {
                active.insert(i);
            }
            LogPayload::Undo => {
                if let Some(&j) = active.iter().next_back() {
                    active.remove(&j);
                    redo_buf.push(j);
                }
            }
            LogPayload::Redo => {
                if let Some(j) = redo_buf.pop() {
                    active.insert(j);
                }
            }
            LogPayload::NewSession { .. } | LogPayload::Checkpoint(_) => {}
        }
    }

    active
}

/// Compute the set of branches transitively merged into `view_branch` as of
/// `as_of`, considering only `MergeBranch` entries that are *live* under each
/// originating session's per-session resolution.
///
/// Placeholder: this is wired up in Task 13. For now, returns the singleton
/// containing `view_branch`.
#[allow(dead_code, reason = "consumed by create_view in the next task")]
pub fn compute_visible_branches(
    log: &[LogEntry],
    view_branch: BranchId,
    as_of: u64,
    session_branch: &HashMap<SessionId, BranchId>,
    live_merges: &[&LogEntry],
) -> HashSet<BranchId> {
    let _ = (log, as_of, session_branch, live_merges);
    let mut visible = HashSet::new();
    visible.insert(view_branch);
    visible
}
