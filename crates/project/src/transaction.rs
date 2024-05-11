use std::{fmt::Debug, hash::Hash};

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
