//! Data and state management.
//!
//! Each data section is described by a [`module::Module`]. Each data section is a member of a single document and
//! has a project wide unique Uuid, which is stable across renames.

// Public modules and re-exports
pub mod transaction;
pub use session::{DataSession, Snapshot};

// Internal modules
pub(crate) mod internal;
pub(crate) mod session;
