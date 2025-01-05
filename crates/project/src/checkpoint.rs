//! Project Checkpoints.
//!
//! Checkpoints are used to give a point in the log [`crate::Project::log`] a nameable identifier.
//!
//! Uses for this include:
//! - Associate a revision with a specific point in time.
//! - Split a Design Task into multiple chronological steps.
//!
//! It must however be noted that generating a [`crate::ProjectView`] until a specific [`CheckpointId`]
//! will NOT guarantee the same [`crate::ProjectView`] due to possible insertion of new [`crate::ProjectLogEntry`] before the
//! checkpoint when an offline user reconnects and uploads its changes.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier to a point in the [`crate::Project::log`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct CheckpointId(Uuid);
