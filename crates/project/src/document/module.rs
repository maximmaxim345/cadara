//! Module with the [`Module`] trait.
use crate::transaction::{DocumentTransaction, ReversibleDocumentTransaction};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Modules are the main building blocks of a document in `CADara`.
///
/// Each document is represented by a single module, which defines the data structure of the document and its behaviours.
/// A module is responsible for defining the following aspects of a document:
/// - Data Structure: The data that is stored for each document, separated into four categories.
/// - Transactions: How transactions are applied to each of the four data structures, which is used to modify the document.
/// - Links: How links and dependencies are handled between documents.
// TODO: look into while we require Clone and Default on this
pub trait Module: Clone + Default + Debug + 'static {
    /// Data structure used for persistent storage of the document.
    ///
    /// # Notes
    /// - This data is saved to disk and should be enough to load the document from disk.
    type DocumentData: ReversibleDocumentTransaction
        + Clone
        + Default
        + Debug
        + PartialEq
        + Serialize
        + for<'a> Deserialize<'a>;
    /// Data structure used for persistent storage of the user's state.
    ///
    /// This data is saved to disk, but should not be necessary to load the document from disk.
    ///
    /// # Notes
    /// - This data is not shared between different users.
    type UserData: ReversibleDocumentTransaction
        + Clone
        + Default
        + Debug
        + PartialEq
        + Serialize
        + for<'a> Deserialize<'a>;
    /// Data structure used for data which persists until the user closes the session.
    ///
    /// # Notes
    /// - This data is not shared between users.
    /// - This data is not saved to disk.
    type SessionData: DocumentTransaction + Clone + Default + Debug + PartialEq;
    /// Data structure used for data, which is shared between all sessions/users.
    ///
    /// # Notes
    /// - This data will be synchronized between users.
    /// - This data is not saved to disk.
    type SharedData: DocumentTransaction
        + Clone
        + Default
        + Debug
        + PartialEq
        + Serialize
        + for<'a> Deserialize<'a>;

    /// Returns the human-readable name of the module.
    fn name() -> String;
    /// Returns the static [`Uuid`] associated with the module.
    ///
    /// # Returns
    /// The [`Uuid`] associated with the module.
    /// Must be unique for each module.
    fn uuid() -> Uuid;
}
