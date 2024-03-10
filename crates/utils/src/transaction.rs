/// A trait for transactions that can be applied to an object.
///
/// Implements the Command pattern.
/// If the transaction is reversible, it should implement the [`ReversibleTransaction`] trait too.
pub trait Transaction {
    /// The type of arguments required to apply the transaction.
    type Args: Clone;
    /// The type of error that can occur when applying the transaction.
    type Error;
    /// The type of the successful output of the transaction.
    type Output;

    /// Applies the transaction to the object.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// The output of the transaction.
    ///
    /// # Errors
    ///
    /// If the transaction fails, an error should be returned.
    ///
    /// # Notes
    /// - This function is pure, meaning it does not have side effects and will always produce the same output
    ///   and leave the object in the same state when called with the same arguments.
    /// - This function should not alter the object state if an error occurs.
    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}

/// A trait for transactions that can be reversed.
pub trait ReversibleTransaction: Transaction {
    /// The type of data required to undo the transaction.
    type UndoData;

    /// Applies the transaction and returns the necessary data to undo it.
    ///
    /// # Arguments
    /// * `args` - The arguments needed to apply the transaction.
    ///
    /// # Returns
    /// The output of the transaction.
    ///
    /// # Errors
    ///
    /// If the transaction fails, an error should be returned.
    ///
    /// # Notes
    /// - This function is pure, meaning it does not have side effects and will always produce the same output
    ///   and leave the object in the same state when called with the same arguments.
    /// - This function should not alter the object state if an error occurs.
    fn apply(&mut self, args: Self::Args) -> Result<(Self::Output, Self::UndoData), Self::Error>;

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
