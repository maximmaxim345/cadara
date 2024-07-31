pub mod internal;

use super::{
    internal::{AppliedTransaction, InternalData, TransactionState, UndoUnit, UndoneTransaction},
    transaction,
};
use internal::InternalDataSession;
use module::{DataTransaction, Module, ReversibleDataTransaction};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use utils::Transaction;

/// Represents a snapshot of a data sections state in a session.
///
/// A [`Snapshot`] encapsulates the state of a data section at a specific point in time during a session.
/// It includes both persistent and non-persistent data.
/// TODO: add note saying this is main way to access data <- if true
///
/// retrieved using [`DataSession::snapshot`].
///
/// [`DataSession::snapshot`]: crate::data::DataSession::snapshot
#[derive(Clone, Default, Debug, PartialEq, Hash)]
pub struct Snapshot<M: Module> {
    /// The persistent data.
    pub persistent: M::PersistentData,
    /// The persistent user-specific data.
    pub persistent_user: M::PersistentUserData,
    /// The non-persistent user-specific data.
    pub session: M::SessionData,
    /// The non-persistent data shared among users.
    pub shared: M::SharedData,
}

/// Represents an interactive session of a document within a project.
///
/// A [`DataSession`] encapsulates the state of an open document that is part of a [`Project`].
/// It maintains a copy of the document's state, allowing for concurrent editing and individual
/// management of persistent and non-persistent data.
///
/// Modifications to the document are made by passing [`Transaction`]s that describe the desired changes.
///
/// [`Project`]: crate::Project
#[derive(Clone, Debug)]
pub struct DataSession<M: Module> {
    /// The internal implementation of this session.
    pub(crate) session: Rc<RefCell<InternalDataSession<M>>>,
    pub(crate) data_model_ref: Weak<RefCell<InternalData<M>>>,
}

impl<M: Module> DataSession<M> {
    /// Captures the current state of the session in a snapshot.
    ///
    /// A snapshot includes all relevant session data, such as persistent data and
    /// data shared among users for the current session.
    ///
    /// # Returns
    /// A `Snapshot<M>` that encapsulates the session's current state.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot<M> {
        let session = self.session.borrow();
        Snapshot {
            persistent: session.persistent.clone(),
            persistent_user: session.persistent_user.clone(),
            session: session.session_data.clone(),
            shared: session.shared_data.clone(),
        }
    }

    // TODO: add doc
    fn apply_session(
        &self,
        args: <M::SessionData as DataTransaction>::Args,
    ) -> Result<<M::SessionData as DataTransaction>::Output, transaction::SessionApplyError<M>>
    {
        let mut internal = self.session.borrow_mut();
        // We do not need to replicate the transaction to other sessions.
        internal.session_data.apply(args).map_or_else(
            |error| {
                Result::Err(transaction::SessionApplyError::TransactionFailure(
                    transaction::TransactionError::Session(error),
                ))
            },
            Result::Ok,
        )
    }

    /// Retrieves a list of all transactions along with the position of the next transaction to be redone.
    ///
    /// This function returns a list of human-readable strings describing the transactions
    /// and an index indicating the position of the next transaction to be redone in the transaction history.
    /// Transactions before this index have been applied and can be undone,
    /// while transactions at or after this index have been undone and can be redone.
    ///
    /// If the index points outside the list (i.e., it's equal to the length of the list),
    /// it means that all transactions have been applied and none can be redone.
    ///
    /// To undo or redo a specific number of transactions, use the `undo` and `redo` functions
    /// with the number of steps to move from the current position.
    ///
    /// # Returns
    ///
    /// A tuple containing a vector of strings and an index:
    ///
    /// * The vector contains descriptions of all transactions (that can be undone or were undone) in the order they were applied.
    /// * The index indicates the position of the next transaction to be redone in the transaction history.
    ///   If the index points outside the list, all transactions have been applied.
    ///
    /// # Panics
    ///
    /// This function is not expected to panic under normal circumstances.
    #[must_use]
    pub fn undo_redo_list(&self) -> (Vec<String>, usize) {
        let session_uuid = self.session.borrow().session_uuid;
        let ref_cell = self.data_model_ref.upgrade().unwrap();
        let internal_data = ref_cell.borrow();
        let history = &internal_data.transaction_history;

        let mut undo_list = Vec::new();
        let mut position = 0;
        let mut undone_found = false;

        for history_state in history {
            if history_state.session != session_uuid {
                continue;
            }
            undo_list.push(history_state.name.clone());
            match &history_state.state {
                // We hide failed transactions from the user, since current the user can't do anything about them
                TransactionState::Applied(_) | TransactionState::Failed(_) => {
                    assert!(
                        !undone_found,
                        "Found applied transaction after an undone transaction"
                    );
                    position = undo_list.len(); // Update position to the end of the list
                }
                TransactionState::Undone(_) => {
                    if !undone_found {
                        undone_found = true; // Mark that we've found an undone transaction
                    }
                }
            }
        }

        (undo_list, position)
    }

    /// Reverts the last `n` transactions applied to this session.
    ///
    /// This function undoes the last `n` undoable transactions that were applied through this session.
    /// It only affects the persistent and user data sections.
    ///
    /// When a transaction is undone, the state of all other sessions and the project is updated accordingly.
    /// If another session applied a transaction after the one being undone, the system will attempt to reapply all subsequent transactions that are still valid.
    /// If a transaction is invalid (e.g., an 'edit sketch' transaction on a now non-existent sketch), that transaction is ignored.
    /// However, transactions following the invalid one will still be applied if they are valid.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of transactions to undo.
    ///
    /// # Panics
    ///
    /// This function is not expected to panic under normal circumstances.
    #[allow(clippy::too_many_lines)]
    pub fn undo(&self, n: usize) {
        enum UndoData<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
            Persistent(D::UndoData),
            PersistentUser(U::UndoData),
            WasFailed,
        }

        struct UndoAction<M: Module> {
            index: usize,
            queued: UndoData<M::PersistentData, M::PersistentUserData>,
            should_redo: bool,
        }

        let session_uuid = self.session.borrow().session_uuid;
        let ref_cell = self.data_model_ref.upgrade().unwrap();
        let mut internal_data = ref_cell.borrow_mut();

        // This algorithm is a bit tricky, since multi user editing is allowed.
        // Here is an explanation of how it works:
        // - we can keep all transactions before the earliest transaction to undo
        // - remember the index, where the earliest undo needs to be performed to each data section
        //   (a separate data section is for example persistent data (for all users), user data of user 1, user data of user 2, ...)
        //   (if the section is not touched, set the index to the end of the list)
        // - update all transactions until the earliest to undo as follows, in reverse order:
        //   - if we want to undo it: undo the transaction
        //   - if we are after the earliest to undo for this data section: undo it, mark it for later redo
        //   - otherwise: keep it as is
        // - update all transactions after the earliest to undo as follows, in order:
        //   - if it is undone and marked for later redo or was Failed:
        //     - try to redo it
        //     - if that fails: mark it as Failed
        //   - if it is already applied: keep it as is

        // This is an example of how the history might look like:
        // In this example, session 2 wants to undo 2 transactions
        // (t) marks the order of the algorithm

        // |               before undo                 |               while undoing
        // - Session 1: Applied - some data transaction <| All transactions before and including this one can be kept as is
        // - Session 2: Applied - some data transaction <(4) undo    << This is a transaction we want to undo, earliest data
        // - Session 1: Applied - some data transaction <(3) undo and mark <(5) try redo
        // - Session 1: Applied - some usr1 transaction
        // - Session 2: Applied - some usr2 transaction <(2) undo    << This is a transaction we want to undo, earliest usr2
        // - Session 2: Undone  - some usr2 transaction
        // - Session 1: Applied - some data transaction <(1) undo and mark <(6) try redo

        // |               after undo                  |
        // - Session 1: Applied - some data transaction
        // - Session 2: Undone  - some data transaction
        // - Session 1: Applied - some data transaction
        // - Session 1: Applied - some usr1 transaction
        // - Session 2: Undone  - some usr2 transaction
        // - Session 2: Undone  - some usr2 transaction
        // - Session 1: Failed  - some data transaction << If a transaction can't be redone, mark it as failed

        // find the earliest transaction of each section, that we want to undo
        let mut earliest_undo_data = internal_data.transaction_history.len();
        let mut earliest_undo_user = internal_data.transaction_history.len();
        let mut undos_left = n;
        // We only want to undo transactions that were applied by this session
        for (index, history_state) in internal_data
            .transaction_history
            .iter()
            .enumerate()
            .rev()
            .filter(|h| h.1.session == session_uuid)
        {
            if let TransactionState::Applied(transaction) = &history_state.state {
                match transaction {
                    super::internal::AppliedTransaction::Persistent(_) => {
                        earliest_undo_data = index;
                    }
                    super::internal::AppliedTransaction::PersistentUser(_) => {
                        earliest_undo_user = index;
                    }
                }
                undos_left -= 1;
            }
            if undos_left == 0 {
                // Everything before this can be kept as is
                break;
            }
        }

        // This is the list of all transactions that need to be undone (and possibly redone)
        let mut queued_undos: Vec<UndoAction<M>> = Vec::new();
        // In reverse order, update as described above
        let mut undos_left = n;
        for (index, history_state) in internal_data
            .transaction_history
            .iter_mut()
            .enumerate()
            .rev()
        {
            match &history_state.state {
                TransactionState::Failed(_) => {
                    if history_state.session == session_uuid {
                        // This is a transaction the user requested to undo
                        // Since it was failed, we only need to mark it as undone
                        queued_undos.push(UndoAction {
                            index,
                            queued: UndoData::WasFailed,
                            should_redo: false,
                        });
                        undos_left -= 1;
                    } else {
                        // We want to redo this transaction if possible
                        queued_undos.push(UndoAction {
                            index,
                            queued: UndoData::WasFailed,
                            should_redo: true,
                        });
                    };
                }
                TransactionState::Applied(transaction) => {
                    if history_state.session == session_uuid {
                        // This is a transaction the user requested to undo
                        let undo = match transaction {
                            AppliedTransaction::Persistent(undo_unit) => {
                                UndoData::Persistent(undo_unit.undo_data.clone())
                            }
                            AppliedTransaction::PersistentUser(undo_unit) => {
                                UndoData::PersistentUser(undo_unit.undo_data.clone())
                            }
                        };
                        queued_undos.push(UndoAction {
                            index,
                            queued: undo,
                            should_redo: false,
                        });
                        undos_left -= 1;
                    } else {
                        // Test if it is after the earliest to undo, if yes undo and mark it
                        match transaction {
                            AppliedTransaction::Persistent(undo_unit) => {
                                if index > earliest_undo_data {
                                    queued_undos.push(UndoAction {
                                        index,
                                        queued: UndoData::Persistent(undo_unit.undo_data.clone()),
                                        should_redo: true,
                                    });
                                }
                            }
                            AppliedTransaction::PersistentUser(undo_unit) => {
                                if index > earliest_undo_user {
                                    queued_undos.push(UndoAction {
                                        index,
                                        queued: UndoData::PersistentUser(
                                            undo_unit.undo_data.clone(),
                                        ),
                                        should_redo: true,
                                    });
                                }
                            }
                        }
                    }
                }
                TransactionState::Undone(_) => {
                    // Leave it as is
                }
            }
            if undos_left == 0 {
                // We are done, since we undid all transactions we wanted to
                break;
            }
        }

        // Execute all queued undos
        for UndoAction {
            index,
            queued,
            should_redo,
        } in &queued_undos
        {
            match queued {
                UndoData::Persistent(undo_data) => {
                    internal_data.persistent_data.undo(undo_data.clone());
                    if !should_redo {
                        // Now mark it as Undone, since we don't want to redo it
                        let state = &mut internal_data.transaction_history[*index].state;
                        *state = if let TransactionState::Applied(AppliedTransaction::Persistent(
                            undo_unit,
                        )) = state
                        {
                            // Update the state
                            TransactionState::Undone(UndoneTransaction::Persistent(
                                undo_unit.args.clone(),
                            ))
                        } else {
                            // This should never happen, since we only queue Applied transactions
                            panic!("Found undone transaction marked for redo");
                        };
                    }
                    // The rest is updated in the redo loop
                }
                UndoData::PersistentUser(undo_data) => {
                    internal_data.user_data.undo(undo_data.clone());
                    if !should_redo {
                        // Now mark it as Undone, since we don't want to redo it
                        let state = &mut internal_data.transaction_history[*index].state;
                        *state = if let TransactionState::Applied(
                            AppliedTransaction::PersistentUser(undo_unit),
                        ) = state
                        {
                            // Update the state
                            TransactionState::Undone(UndoneTransaction::PersistentUser(
                                undo_unit.args.clone(),
                            ))
                        } else {
                            // This should never happen, since we only queue Applied transactions
                            panic!("Found undone transaction marked for redo");
                        };
                    }
                    // The rest is updated in the redo loop
                }
                UndoData::WasFailed => {
                    let is_this_session =
                        internal_data.transaction_history[*index].session == session_uuid;
                    if is_this_session {
                        let state = &mut internal_data.transaction_history[*index].state;
                        // Just change the state from Failed to Undone
                        if let TransactionState::Failed(transaction) = state {
                            *state = TransactionState::Undone(transaction.clone());
                        } else {
                            // This should never happen, since we only queue Failed transactions
                            panic!("Found undone/applied transaction marked as failed");
                        }
                    }
                }
            }
        }

        // Now redo all transactions that were undone and marked for later redo
        // This loop runs in order (since the queue was constructed in rev order)
        for UndoAction { index, queued, .. } in queued_undos.iter().rev().filter(|u| u.should_redo)
        {
            // This is a transaction we want to redo
            match queued {
                UndoData::PersistentUser(_) | UndoData::Persistent(_) | UndoData::WasFailed => {
                    // We redo the transaction and update the record in the history
                    internal_data.transaction_history[*index].state = match internal_data
                        .transaction_history[*index]
                        .state
                        .clone()
                    {
                        TransactionState::Applied(transaction) => {
                            // It is still marked as applied, but it is undone
                            // Try to redo it
                            match transaction {
                                AppliedTransaction::Persistent(undo_unit) => {
                                    let result = ReversibleDataTransaction::apply(
                                        &mut internal_data.persistent_data,
                                        undo_unit.args.clone(),
                                    );
                                    match result {
                                        Ok((_output, undo_data)) => TransactionState::Applied(
                                            AppliedTransaction::Persistent(UndoUnit {
                                                undo_data,
                                                args: undo_unit.args,
                                            }),
                                        ),
                                        Err(_error) => TransactionState::Failed(
                                            UndoneTransaction::Persistent(undo_unit.args),
                                        ),
                                    }
                                }
                                AppliedTransaction::PersistentUser(undo_unit) => {
                                    let result = ReversibleDataTransaction::apply(
                                        &mut internal_data.user_data,
                                        undo_unit.args.clone(),
                                    );
                                    match result {
                                        Ok((_output, undo_data)) => TransactionState::Applied(
                                            AppliedTransaction::PersistentUser(UndoUnit {
                                                undo_data,
                                                args: undo_unit.args,
                                            }),
                                        ),
                                        Err(_error) => TransactionState::Failed(
                                            UndoneTransaction::PersistentUser(undo_unit.args),
                                        ),
                                    }
                                }
                            }
                        }
                        TransactionState::Failed(transaction) => {
                            // We want to redo this transaction if the data section is touched
                            // (if it is not touched, we don't need to redo it)
                            match transaction {
                                UndoneTransaction::Persistent(args) => {
                                    if *index < earliest_undo_data {
                                        // Since we didn't touch the data section, redoing it would fail again
                                        TransactionState::Failed(UndoneTransaction::Persistent(
                                            args,
                                        ))
                                    } else {
                                        let result = ReversibleDataTransaction::apply(
                                            &mut internal_data.persistent_data,
                                            args.clone(),
                                        );
                                        match result {
                                            Ok((_output, undo_data)) => TransactionState::Applied(
                                                AppliedTransaction::Persistent(UndoUnit {
                                                    undo_data,
                                                    args,
                                                }),
                                            ),
                                            Err(_error) => TransactionState::Failed(
                                                UndoneTransaction::Persistent(args),
                                            ),
                                        }
                                    }
                                }
                                UndoneTransaction::PersistentUser(args) => {
                                    if *index < earliest_undo_user {
                                        // Since we didn't touch the data section, redoing it would fail again
                                        TransactionState::Failed(UndoneTransaction::PersistentUser(
                                            args,
                                        ))
                                    } else {
                                        let result = ReversibleDataTransaction::apply(
                                            &mut internal_data.user_data,
                                            args.clone(),
                                        );
                                        match result {
                                            Ok((_output, undo_data)) => TransactionState::Applied(
                                                AppliedTransaction::PersistentUser(UndoUnit {
                                                    undo_data,
                                                    args,
                                                }),
                                            ),
                                            Err(_error) => TransactionState::Failed(
                                                UndoneTransaction::PersistentUser(args),
                                            ),
                                        }
                                    }
                                }
                            }
                        }
                        TransactionState::Undone(_transaction) => {
                            // This should never happen, since this method should not explicitly redo transactions
                            panic!("Found undone transaction marked for redo");
                        }
                    };
                }
            }
        }

        // Copy the data to all sessions
        for session in &internal_data.sessions {
            let session = session.1.upgrade().unwrap();
            session.borrow_mut().persistent = internal_data.persistent_data.clone();
            session.borrow_mut().persistent_user = internal_data.user_data.clone();
        }
    }

    /// Redoes a specified number of previously undone transactions in this session.
    ///
    /// This function redoes the last `n` transactions that were previously undone in this session.
    /// The redo operation applies only to the persistent and persistent user data sections.
    ///
    /// When a transaction is redone, the state of all other sessions and the project is updated accordingly.
    /// If any other sessions applied transactions after the transaction that is being redone,
    /// the system will attempt to reapply all those transactions which are still valid.
    /// For instance, if a transaction from another user became invalid due to an undo operation,
    /// calling `redo` will make it valid again.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of transactions to redo. If `n` is greater than the number of transactions that can be redone, all possible transactions are redone.
    ///
    /// # Panics
    ///
    /// This function is not expected to panic under normal circumstances.
    #[allow(clippy::too_many_lines)]
    pub fn redo(&self, n: usize) {
        // enum RedoData<D: ReversibleDataTransaction, U: ReversibleDataTransaction> {
        //     Persistent(D::UndoData),
        //     PersistentUser(U::UndoData),
        //     WasFailed,
        // }
        enum ActionType {
            Recompute,
            NoActionRequired,
        }

        struct Action {
            index: usize,
            action_type: ActionType,
            // queued: RedoData<M::PersistentData, M::PersistentUserData>,
            // should_redo: bool,
        }

        let session_uuid = self.session.borrow().session_uuid;
        let ref_cell = self.data_model_ref.upgrade().unwrap();
        let mut internal_data = ref_cell.borrow_mut();
        // let _history = &internal_data.transaction_history;

        // This algorithm is for redoing transactions in a multi-user editing system.
        // It complements the undo algorithm described in the undo() method.
        // Here's how the redo process works:
        // - update all transactions in reverse order:
        //   - if we reach an applied or failed transaction from this session:
        //     - break the loop, save the next index for the next loop
        //   - if we reach an failed transaction:
        //     - mark it for later redo
        //   - if we reach an applied from another session:
        //     - undo it, mark it for later redo
        // - set redone count to 0
        // - second loop, this time in order, we start at the one after the last iteration of the first loop
        //   - if it is an undone transaction from this session and
        //     redone count is less than the number of transactions we want to redo:
        //     - try to redo it
        //     - increase the redone count
        //   - if it is marked for later redo:
        //     - try to redo it

        // We optimize this algorithm a bit by not undoing in the first loop,
        // but just noting that we need to undo it
        // We than filter them like this:
        // - set redone count to 0
        // - loop, in order, we start at the one after the last iteration of the first loop
        //   - if it is an undone transaction from this session and
        //     redone count is less than the number of transactions we want to redo:
        //     - mark this section as touched
        //     - increase the redone count
        //   - if it is marked for later redo:
        //     - if the section is not touched: delete it from undone and redone list
        // This way we don't have to undo and redo transactions
        // where nothing redone happened in between

        // Here's the example from the undo() method, session 2 wants to again redo the 2 transactions:
        // |                 before redo               |              while redoing
        //                                              <(4) we ran the first loop until here
        // - Session 1: Applied - some data transaction    <(5) we start the second loop here
        // - Session 2: Undone  - some data transaction    <(6) redo, increase redone count
        // - Session 1: Applied - some data transaction <(3) undo and mark <(7) try redo
        // - Session 1: Applied - some usr1 transaction <(2) undo and mark <(8) try redo
        // - Session 2: Undone  - some usr2 transaction    <(9) redo, increase redone count
        // - Session 2: Undone  - some usr2 transaction    <(10) do nothing, since redone count is already 2
        // - Session 1: Failed  - some data transaction <(1) mark for redo <(11) try redo

        // |                 after  redo               |
        // - Session 1: Applied - some data transaction
        // - Session 2: Applied - some data transaction
        // - Session 1: Applied - some data transaction
        // - Session 1: Applied - some usr1 transaction
        // - Session 2: Applied - some usr2 transaction
        // - Session 2: Undone  - some usr2 transaction
        // - Session 1: Applied - some data transaction

        // This is the list of all actions that need to be performed (like undo or undo+redo)
        // We don't yet call undo, so that we can optimize redundant undo/redo actions
        let mut queued_actions: Vec<Action> = Vec::new();
        let mut next_iteration = 0;
        // In reverse order, update as described above
        for (index, history_state) in internal_data
            .transaction_history
            .iter_mut()
            .enumerate()
            .rev()
        {
            match &history_state.state {
                TransactionState::Applied(_) | TransactionState::Failed(_) => {
                    if history_state.session == session_uuid {
                        // This is from our session and we are done, since we found the last applied transaction
                        // In the case of a Failed transaction:
                        // we won't retry to redo it, since it failed and the data didn't change before it.
                        // it would fail again!
                        next_iteration = index + 1;
                        break;
                    }
                    // We need to undo this transaction, since we might conflict with it
                    queued_actions.push(Action {
                        index,
                        action_type: ActionType::Recompute,
                    });
                }
                TransactionState::Undone(_) => {
                    // Undone transactions should be ignored in this step
                }
            }
        }

        let mut redo_left = n;
        let mut data_touched = false;
        let mut user_touched = false;
        // Optimize redundant undo/redo actions
        for index in next_iteration..internal_data.transaction_history.len() {
            // TODO: replace find with something more efficient (we know that the list is sorted)
            let action = queued_actions.iter_mut().find(|a| a.index == index);
            let history_state = &mut internal_data.transaction_history[index];
            match &history_state.state {
                TransactionState::Undone(transaction) => {
                    if (history_state.session == session_uuid) && redo_left > 0 {
                        // This is an undone transaction that we want to redo
                        redo_left -= 1;
                        // Mark the data section as changed
                        match transaction {
                            UndoneTransaction::Persistent(_) => data_touched = true,
                            UndoneTransaction::PersistentUser(_) => user_touched = true,
                        }
                    }
                }
                TransactionState::Applied(transaction) => {
                    if let Some(action) = action {
                        // This is a transaction that is planned for undo+redo
                        match transaction {
                            AppliedTransaction::Persistent(_) => {
                                if !data_touched {
                                    // The data section is not affected by the redo() call
                                    // so we don't need to execute the action
                                    action.action_type = ActionType::NoActionRequired;
                                }
                            }
                            AppliedTransaction::PersistentUser(_) => {
                                if !user_touched {
                                    action.action_type = ActionType::NoActionRequired;
                                }
                            }
                        }
                    }
                }
                TransactionState::Failed(transaction) => {
                    if let Some(action) = action {
                        // This is a failed transaction that is planned for redo
                        match transaction {
                            UndoneTransaction::Persistent(_) => {
                                if !data_touched {
                                    // The data section is not affected by the redo() call
                                    // redoing it would fail again
                                    action.action_type = ActionType::NoActionRequired;
                                }
                            }
                            UndoneTransaction::PersistentUser(_) => {
                                if !user_touched {
                                    action.action_type = ActionType::NoActionRequired;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Now we can execute the undo actions
        for action in &queued_actions {
            let index = action.index;
            match action.action_type {
                ActionType::Recompute => {
                    // Execute the undo and save the new state
                    // TODO: saving the state may not be necessary, since we later change it again
                    internal_data.transaction_history[index].state =
                        match internal_data.transaction_history[index].state.clone() {
                            TransactionState::Applied(transaction) => match transaction {
                                AppliedTransaction::Persistent(undo_unit) => {
                                    internal_data.persistent_data.undo(undo_unit.undo_data);
                                    TransactionState::Undone(UndoneTransaction::Persistent(
                                        undo_unit.args,
                                    ))
                                }
                                AppliedTransaction::PersistentUser(undo_unit) => {
                                    internal_data.user_data.undo(undo_unit.undo_data);
                                    TransactionState::Undone(UndoneTransaction::PersistentUser(
                                        undo_unit.args,
                                    ))
                                }
                            },
                            TransactionState::Undone(_) => {
                                // This should never happen, since we only queue Applied or Failed transactions
                                // for recomputing
                                panic!("Found undone transaction marked for redo");
                            }
                            TransactionState::Failed(transaction) => {
                                // Failed transactions are already not applied, just mark them as undone
                                TransactionState::Undone(transaction)
                            }
                        }
                }
                ActionType::NoActionRequired => {
                    // We can leave it as is
                }
            }
        }

        let mut redo_left = n;
        // We now want to redo all transactions that the user requested and that are marked for redo
        for index in next_iteration..internal_data.transaction_history.len() {
            // TODO: again, replace find with something more efficient (we know that the list is sorted)
            let action = queued_actions.iter_mut().find(|a| a.index == index);
            let is_current_session =
                internal_data.transaction_history[index].session == session_uuid;

            if let TransactionState::Undone(transaction) =
                internal_data.transaction_history[index].state.clone()
            {
                // If true, we want to redo it
                let is_user_requested = is_current_session && redo_left > 0;
                // If true, it was marked for redo
                let marked_for_recompute = action.map_or(false, |action| {
                    matches!(action.action_type, ActionType::Recompute)
                });

                if is_user_requested {
                    // This is a transaction, that the user requested to redo
                    // count it as a done
                    redo_left -= 1;
                }

                if is_user_requested || marked_for_recompute {
                    // We try to redo it, and update the state accordingly
                    internal_data.transaction_history[index].state = match transaction {
                        UndoneTransaction::Persistent(args) => {
                            match ReversibleDataTransaction::apply(
                                &mut internal_data.persistent_data,
                                args.clone(),
                            ) {
                                Ok((_, undo_data)) => TransactionState::Applied(
                                    AppliedTransaction::Persistent(UndoUnit { undo_data, args }),
                                ),
                                Err(_) => {
                                    TransactionState::Failed(UndoneTransaction::Persistent(args))
                                }
                            }
                        }
                        UndoneTransaction::PersistentUser(args) => {
                            match ReversibleDataTransaction::apply(
                                &mut internal_data.user_data,
                                args.clone(),
                            ) {
                                Ok((_, undo_data)) => {
                                    TransactionState::Applied(AppliedTransaction::PersistentUser(
                                        UndoUnit { undo_data, args },
                                    ))
                                }
                                Err(_) => TransactionState::Failed(
                                    UndoneTransaction::PersistentUser(args),
                                ),
                            }
                        }
                    };
                }
            }
        }

        // Copy the data to all sessions
        for session in &internal_data.sessions {
            let session = session.1.upgrade().unwrap();
            session.borrow_mut().persistent = internal_data.persistent_data.clone();
            session.borrow_mut().persistent_user = internal_data.user_data.clone();
        }
    }
}

impl<M: Module> Transaction for DataSession<M> {
    type Args = transaction::TransactionArgs<M>;
    type Error = transaction::SessionApplyError<M>;
    type Output = transaction::TransactionOutput<M>;

    fn apply(&mut self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if let Self::Args::Session(session_args) = args {
            // Session data is not synced with other sessions, so we can just directly apply it
            self.apply_session(session_args)
                .map_or_else(Result::Err, |output| {
                    Ok(transaction::TransactionOutput::Session(output))
                })
        } else {
            // The remaining transaction are applied through the data model
            // This is because they need to be synced with other session.
            let session_uuid = self.session.borrow().session_uuid;
            let ref_cell = &self.data_model_ref.upgrade().unwrap();
            let mut internal_data = ref_cell.borrow_mut();
            match args {
                Self::Args::Persistent(data_args) => internal_data
                    .apply_persistent(data_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::Persistent(output))
                    }),
                Self::Args::PersistentUser(user_args) => internal_data
                    .apply_user(user_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::PersistentUser(output))
                    }),
                Self::Args::Shared(shared_args) => internal_data
                    .apply_shared(&shared_args, session_uuid)
                    .map_or_else(Result::Err, |output| {
                        Ok(transaction::TransactionOutput::Shared(output))
                    }),
                // We already handled this case above
                Self::Args::Session(_) => unreachable!(),
            }
        }
    }
}
