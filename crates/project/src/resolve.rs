//! Pure resolvers for layers 1 and 2 of the project's CRDT architecture.
//!
//! See the crate-level docs for what the layers are.

use crate::branch::BranchId;
use crate::oplog::{LogEntry, LogPayload};
use std::collections::{BTreeSet, HashMap, VecDeque};

/// Walk a session's entries (already lamport-sorted) and return the indices
/// (into `entries`) whose effect is currently active.
///
/// Undo and Redo are inherently scoped to the issuing session: this function
/// receives only one session's entries, so an `Undo` entry can only target
/// `Changes` or `MergeBranch` entries from the same session. Multi-user safety
/// (one user's undo can't touch another's edits) follows for free.
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
    // BFS over reverse-merge edges starting from view_branch. Each BFS state
    // carries the branch and an upper bound on the next hop's lamport: the
    // lamport of the merge that landed us on this branch. The first hop is
    // bounded by `as_of + 1`, so the gate `merge_lamport < upper_bound` plus
    // `merge_lamport <= as_of` collapse into a single comparison.
    //
    // The op must already be present on each source branch at the moment the
    // merge ran, so we also gate `op_lamport < merge_lamport` at every hop.
    // The last hop imports the op when `from == op_branch`.
    let initial_bound = as_of.saturating_add(1);
    let mut frontier: VecDeque<(BranchId, u64)> = VecDeque::new();
    frontier.push_back((view_branch, initial_bound));
    // Track the highest upper bound each branch has been visited with. A new
    // visit is only worth enqueuing if it raises that bound, since a larger
    // bound strictly expands the set of merges reachable from there.
    let mut best_bound: HashMap<BranchId, u64> = HashMap::new();
    best_bound.insert(view_branch, initial_bound);

    while let Some((branch, upper_bound)) = frontier.pop_front() {
        if let Some(merges) = live_merges_by_into.get(&branch) {
            for &(from, merge_lamport) in merges {
                if merge_lamport >= upper_bound {
                    continue;
                }
                if op_lamport >= merge_lamport {
                    continue;
                }
                if from == op_branch {
                    return true;
                }
                let prev = best_bound.get(&from).copied().unwrap_or(0);
                if merge_lamport > prev {
                    best_bound.insert(from, merge_lamport);
                    frontier.push_back((from, merge_lamport));
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
