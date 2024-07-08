#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};
use uuid::Uuid;

/// A trait for transactions that can be applied to data section as defined by the [`Module`] trait.
///
/// Implements the Command pattern.
/// If the transaction is reversible, it should implement the [`ReversibleDocumentTransaction`] trait too.
///
/// [`Module`]: crate::Module
pub trait DocumentTransaction {
    // TODO: add Debug, Clone, ... to these types
    /// The type of arguments required to apply the transaction.
    type Args: Clone + Debug + PartialEq + Hash;
    /// The type of error that can occur when applying the transaction.
    type Error: Clone + Debug + PartialEq;
    /// The type of the successful output of the transaction.
    type Output: Clone + Debug + PartialEq;

    /// Applies the transaction to the object.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// A result containing either the output of the transaction or an error.
    ///
    /// # Errors
    /// If the transaction cannot be applied, an error variant will be returned. This error contains
    /// information about why the transaction failed.
    ///
    /// # Notes
    /// - This function is pure, meaning it does not have side effects and will always produce the same output
    ///   and leave the object in the same state when called with the same arguments.
    /// - This function should not alter the object state if an error occurs.
    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error>;

    /// Applies the transaction without performing any checks.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// The output of the transaction.
    ///
    /// # Errors
    /// If the transaction cannot be applied, an error variant will be returned. This error contains
    /// information about why the transaction failed.
    ///
    /// # Panics
    /// Panics if the transaction fails, use `apply` instead if you want to handle errors.
    ///
    /// # Notes
    /// - This function assumes that all preconditions are met and does not need to perform any validation.
    ///   It is intended for use cases where the caller guarantees the correctness of the arguments,
    ///   by previously calling apply with the same arguments (on a equivalent object).
    /// - This function should otherwise behave the same as apply.
    fn apply_unchecked(&mut self, args: Self::Args) -> Self::Output {
        self.apply(args)
            .unwrap_or_else(|_| panic!("Unchecked transaction failed with error"))
    }

    /// Returns the name of the transaction for the undo history.
    ///
    /// # Returns
    /// The name of the transaction, should be a short string, ideally max 20 characters.
    fn undo_history_name(args: &Self::Args) -> String;
}

/// A trait for transactions that can be reversed.
pub trait ReversibleDocumentTransaction: DocumentTransaction {
    /// The type of data required to undo the transaction.
    type UndoData: Clone + Debug + PartialEq + Hash;

    /// Applies the transaction and returns the necessary data to undo it.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// A result containing either a tuple of the transaction output and undo data, or an error.
    ///
    /// # Errors
    /// If the transaction cannot be applied, an error variant will be returned. This error contains
    /// information about why the transaction failed.
    ///
    /// # Notes
    /// - This function is pure, meaning it does not have side effects and will always produce the same output
    ///   and leave the object in the same state when called with the same arguments.
    /// - This function should not alter the object state if an error occurs.
    fn apply(&mut self, args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error>;

    /// Applies the transaction without performing any checks and returns the undo data.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// A tuple of the transaction output and undo data.
    ///
    /// # Panics
    /// Panics if the transaction fails, which should never happen.
    ///
    /// # Notes
    /// - This function assumes that all preconditions are met and does not need to perform any validation.
    ///   It is intended for use cases where the caller guarantees the correctness of the arguments,
    ///   by previously calling apply with the same arguments (on a equivalent object).
    /// - This function should otherwise behave the same as apply.
    fn apply_unchecked(&mut self, args: Self::Args) -> (Self::Output, Self::UndoData) {
        ReversibleDocumentTransaction::apply(self, args)
            .unwrap_or_else(|_| panic!("Unchecked transaction failed with error"))
    }

    /// Undoes the transaction using the provided undo data.
    ///
    /// # Arguments
    /// * `undo_data` - The undo data returned by a previous call to `apply` or `apply_unchecked`.
    ///
    /// # Notes
    /// - This function should restore the object to the state it was in before the transaction was applied.
    /// - This function is pure, therefore when called on a equivalent object with the same undo data,
    ///   it should always produce the same output and leave the object in the same state.
    fn undo(&mut self, undo_data: Self::UndoData);
}

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
