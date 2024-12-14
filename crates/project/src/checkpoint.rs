//! Project Checkpoints.
//!
//! Checkpoints are used to give an point in the log [`Project::log`] a namable identifier.
//!
//! Uses for this include:
//! - Associate a revision with a specific point in time.
//! - Split a Design Task into multiple chronological steps.
//!
//! It must however be noted that generating a [`ProjectView`] until a specific [`CheckpointId`]
//! will NOT guarantee the same [`ProjectView`] due to possible insertion of new [`ProjectLogEntry`] before the
//! checkpoint when a offline user reconnects and uploads its changes.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier to a point in the [`Project::log`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct CheckpointId(Uuid);
