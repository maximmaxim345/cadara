//! Document data and state management.
//!
//! Each document is represented by a single [`Module`], which defines the data structures of the document
//! and how it implements behaviour like cross document linking.
//! TODO: update this doc when linking is implemented
//!
//! # Linking and Dependencies
//!
//! Links and dependencies are managed through it's parent [`Project`] and can only be created between documents in the same project.
//!
//! [`Project`]: crate::Project

// Public modules and re-exports
pub mod transaction;
pub use module::Module;
pub use session::{Session, Snapshot};

// Internal modules
pub(crate) mod internal;
pub(crate) mod session;

// TODO: split module into a separate crate
mod module;
