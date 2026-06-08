//! Pure resolvers for Layers 1 and 2 of the CRDT design.

use crate::branch::BranchId;
use crate::oplog::{LogEntry, LogPayload};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Walk a session's entries (already lamport-sorted) and return the indices
/// (into `entries`) whose effect is currently active.
///
/// Only `Changes` and `MergeBranch` payloads occupy the active/redo
/// bookkeeping. `Undo` and `Redo` mutate that bookkeeping. Other payloads
/// (`NewSession`, `Checkpoint`) are ignored at this layer.
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

/// Returns true if op `(op_lamport, op_branch)` is visible from `view_branch`
/// at view time `as_of`, given an index of live `MergeBranch` entries.
pub fn op_is_visible(
    op_lamport: u64,
    op_branch: BranchId,
    view_branch: BranchId,
    as_of: u64,
    live_merges_by_into: &HashMap<BranchId, Vec<(BranchId, u64)>>,
) -> bool {
    if op_branch == view_branch {
        return true;
    }
    // BFS from view_branch over reverse-merge edges. Per the spec, EVERY hop
    // along the chain must satisfy op_lamport < merge_lamport (the op was
    // already on the source branch when that merge happened) AND
    // merge_lamport <= as_of. The hop ending at op_branch is the one that
    // imports the op.
    let mut frontier: VecDeque<BranchId> = VecDeque::new();
    frontier.push_back(view_branch);
    let mut seen: HashSet<BranchId> = HashSet::new();
    seen.insert(view_branch);

    while let Some(branch) = frontier.pop_front() {
        if let Some(merges) = live_merges_by_into.get(&branch) {
            for &(from, merge_lamport) in merges {
                if merge_lamport > as_of || op_lamport >= merge_lamport {
                    continue;
                }
                if from == op_branch {
                    return true;
                }
                if seen.insert(from) {
                    frontier.push_back(from);
                }
            }
        }
    }
    false
}

/// Build the `into -> [(from, merge_lamport)]` adjacency from a slice of
/// live `MergeBranch` log entries.
pub fn live_merges_index(live: &[&LogEntry]) -> HashMap<BranchId, Vec<(BranchId, u64)>> {
    let mut out: HashMap<BranchId, Vec<(BranchId, u64)>> = HashMap::new();
    for entry in live {
        if let LogPayload::MergeBranch { from, into } = entry.payload {
            out.entry(into).or_default().push((from, entry.lamport));
        }
    }
    out
}
