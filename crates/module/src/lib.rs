#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};
use uuid::Uuid;

/// A trait for data sections that can be modified by transactions defined by the [`Module`] trait.
///
/// Implements the Command pattern.
///
/// [`Module`]: crate::Module
pub trait DataSection:
    Clone + Default + Debug + PartialEq + Serialize + Send + Sync + for<'a> Deserialize<'a>
{
    /// The type of arguments required to apply the transaction.
    type Args: Clone
        + Debug
        + PartialEq
        + Hash
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + Send
        + Sync;

    /// Applies the transaction to the data section.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Notes
    /// - This function is deterministic: when called with the same arguments, it will always modify the state of the data section in the same way.
    /// - This function is expected to be without side-effects outside of the modification of the object it was called on.
    /// - This function is expected to not error for any valid `args` object.
    // TODO: maybe args should be a ref?
    fn apply(&mut self, args: Self::Args);

    /// Returns the name of the transaction for the undo history.
    ///
    /// # Arguments
    /// * `args` - The arguments for the transaction.
    ///
    /// # Returns
    /// The name of the transaction, should be a short string, ideally max 20 characters.
    fn undo_history_name(args: &Self::Args) -> String;
}

/// Modules are the main building blocks of a project in `CADara`.
///
/// A project is essentially a collection of data sections (grouped into documents), which are represented by modules.
/// Modules define the data structure stored in them and how it can be modified.
///
/// A module is responsible for defining the following aspects of a data section:
/// - Data Structure: The data that is stored for each section, separated into four categories:
///     - `PersistentData`: Data that is saved to disk and should be enough to load the data from disk.
///     - `PersistentUserData`: Data that is saved to disk, but not required to load data.
///     - `SessionData`: Data that persists until the user closes the session and is not saved.
///     - `SharedData`: Data that is shared between all users and is not saved.
/// - Transactions: How transactions are applied to each of the four data structures, which is used to modify the data.
pub trait Module: Clone + Default + Debug + Send + Sync + 'static {
    /// Data structure used for persistent storage of the data.
    ///
    /// # Notes
    /// - This data is saved to disk and should be enough to load the data from disk.
    type PersistentData: DataSection;
    /// Data structure used for persistent storage of the user's state.
    ///
    /// This data is saved to disk, but should not be necessary to load the data from disk.
    ///
    /// # Notes
    /// - This data is not shared between different users.
    type PersistentUserData: DataSection;
    /// Data structure used for data which persists until the user closes the session.
    ///
    /// # Notes
    /// - This data is not shared between users.
    /// - This data is not saved to disk.
    type SessionData: DataSection;
    /// Data structure used for data, which is shared with all sessions/users.
    ///
    /// # Notes
    /// - This data will be sent to all other users.
    /// - This data is not saved to disk.
    type SharedData: DataSection;

    /// Returns the human-readable name of the module.
    fn name() -> String;
    /// Returns the static [`Uuid`] associated with the module.
    ///
    /// # Returns
    /// The [`Uuid`] associated with the module.
    /// Must be unique for each module.
    fn uuid() -> Uuid;
}
