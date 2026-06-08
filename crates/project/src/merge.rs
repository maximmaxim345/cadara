use crate::module_data::ModuleId;
use crate::oplog::{Change, LogEntry, LogPayload};
use crate::user::SessionId;
use crate::{ModuleRegistry, Project, ProjectSource};
use std::collections::HashSet;

#[derive(thiserror::Error, Debug)]
pub enum MergeError {
    #[error("Cannot merge: replicas belong to different projects.")]
    DifferentProject,
    #[error("The module {0} is used in the remote log, but not registered in the registry")]
    UnknownModule(ModuleId),
}

impl Project {
    /// Merge another replica's log into this one. The two `Project`s must
    /// have the same `uuid`. Idempotent: merging an already-merged log is a
    /// no-op.
    ///
    /// # Errors
    /// Returns `MergeError::DifferentProject` if the replicas have different uuids.
    /// Returns `MergeError::UnknownModule` if the remote log references a module
    /// not present in `reg`.
    pub fn merge_remote(&mut self, other: &Self, reg: &ModuleRegistry) -> Result<(), MergeError> {
        if !self.is_same_source_as(other) {
            return Err(MergeError::DifferentProject);
        }

        // Validate that the registry knows every module referenced in the
        // remote log.
        for entry in &other.log {
            check_entry_modules(entry, reg)?;
        }

        // Dedup by (lamport, session): the same op from both replicas is
        // byte-identical.
        let have: HashSet<(u64, SessionId)> =
            self.log.iter().map(|e| (e.lamport, e.session)).collect();

        for e in &other.log {
            if !have.contains(&(e.lamport, e.session)) {
                self.log.push(e.clone());
            }
        }

        if let Some(m) = other.log.iter().map(|e| e.lamport).max() {
            self.lamport_clock = self.lamport_clock.max(m + 1);
        }
        Ok(())
    }
}

fn check_entry_modules(entry: &LogEntry, reg: &ModuleRegistry) -> Result<(), MergeError> {
    if let LogPayload::Changes(changes) = &entry.payload {
        for c in changes {
            match c {
                Change::CreateData { module, .. } if !reg.0.contains_key(module) => {
                    return Err(MergeError::UnknownModule(*module));
                }
                Change::Transaction { args, .. } if !reg.0.contains_key(&args.module) => {
                    return Err(MergeError::UnknownModule(args.module));
                }
                Change::UserTransaction { args, .. } if !reg.0.contains_key(&args.module) => {
                    return Err(MergeError::UnknownModule(args.module));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
