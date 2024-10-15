mod common;
use common::test_module::*;
use data::{DataSession, DataUuid};
use project::data::transaction::TransactionArgs;
use project::*;

fn create_undo_redo_test_setup() -> (
    ProjectSession,
    DataSession<TestModule>,
    DataSession<TestModule>,
    DataUuid,
    Vec<TestTransaction>,
) {
    let project = Project::new("Project".to_string()).create_session();
    let doc = project.create_document();
    let doc = project.open_document(doc).unwrap();
    let data_uuid = doc.create_data::<TestModule>();

    let mut session1 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
    let mut session2 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();

    let transactions = vec![
        (
            1,
            TransactionArgs::Persistent(TestTransaction::SetWord("word_a".to_string())),
        ),
        (
            1,
            TransactionArgs::PersistentUser(TestTransaction::SetWord("word_b".to_string())),
        ),
        (
            2,
            TransactionArgs::Persistent(TestTransaction::SetWord("word_c".to_string())),
        ),
        (
            2,
            TransactionArgs::PersistentUser(TestTransaction::SetWord("word_d".to_string())),
        ),
        (
            1,
            TransactionArgs::Persistent(TestTransaction::SetWord("word_e".to_string())),
        ),
    ];

    // Apply transactions
    for (session_number, transaction) in &transactions {
        let session = match session_number {
            1 => &mut session1,
            2 => &mut session2,
            _ => panic!("Invalid session number"),
        };
        assert!(session.apply(transaction.clone()).is_ok());
    }

    // Now we cast away unneeded information from the transactions list
    let transactions = transactions
        .into_iter()
        .map(|(_, transaction)| match transaction {
            TransactionArgs::Persistent(transaction) => transaction,
            TransactionArgs::PersistentUser(transaction) => transaction,
            TransactionArgs::Session(transaction) => transaction,
            TransactionArgs::Shared(transaction) => transaction,
        })
        .collect();

    // This will copy the internal logging id from the internal document
    // to our sessions. This is needed to inspect the applied undo/apply
    // transactions until we implement a non copy based distribution
    // of the new state.
    // TODO: Remove this when undo/redo doesn't copy
    session1.redo(1);

    let snapshot = session1.snapshot();
    assert_eq!(snapshot.persistent.single_word, "word_e".to_string());
    assert_eq!(snapshot.persistent_user.single_word, "word_d".to_string());

    let (hist, loc) = session1.undo_redo_list();
    assert_eq!(
        hist,
        vec![
            "Set word to word_a".to_string(),
            "Set word to word_b".to_string(),
            "Set word to word_e".to_string(),
        ]
    );
    assert_eq!(loc, 3);

    let (hist, loc) = session2.undo_redo_list();
    assert_eq!(
        hist,
        vec![
            "Set word to word_c".to_string(),
            "Set word to word_d".to_string(),
        ]
    );
    assert_eq!(loc, 2);

    (project, session1, session2, data_uuid, transactions)
}

#[test]
fn test_undo_document_one_user() {
    // Both session are owned by the same user
    let (project, session1, session2, doc_uuid, transactions) = create_undo_redo_test_setup();
    let session_doc_closure = project.open_data::<TestModule>(doc_uuid).unwrap();
    let session_user_closure = project.open_data::<TestModule>(doc_uuid).unwrap();
    // closures for getting a current snapshot of both data sections and the internal log
    // Since all sessions are owned by the same user, the data should be the same
    let document = || session_doc_closure.snapshot().persistent;
    let user = || session_user_closure.snapshot().persistent_user;
    let get_doc_log_and_clear = || {
        let doc_log_uuid = document().logging_uuid;
        let doc_log = get_transaction_log(doc_log_uuid);
        clear_transaction_log(doc_log_uuid);
        doc_log
    };
    let get_user_log_and_clear = || {
        let user_log_uuid = user().logging_uuid;
        let user_log = get_transaction_log(user_log_uuid);
        clear_transaction_log(user_log_uuid);
        user_log
    };

    // The internal undo stack for the document should look like this:
    // (A is applied, U is undone, F is failed, Ar is applied but redone(undone+applied),
    // Fr is failed but redone(undone+applied failed))
    //
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. A - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. A - User(s2): SetWord("word_d")
    // 4. A - Doc(s1): SetWord("word_e")

    get_doc_log_and_clear();
    get_user_log_and_clear();
    session2.undo(1);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. A - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. A - Doc(s1): SetWord("word_e")
    assert_eq!(user().single_word, "word_b".to_string());
    assert_eq!(document().single_word, "word_e".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 3);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(
        get_user_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[3].clone())]
    );

    session1.undo(2);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")
    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 1);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[4].clone())]
    );
    assert_eq!(
        get_user_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[1].clone())]
    );

    // this should not change anything
    session1.undo(0);
    session2.undo(0);
    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 1);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    session2.undo(1);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. U - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")
    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "word_a".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 1);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 0);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[2].clone())]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    // And request 10 undo steps on session 1, should only undo 1
    session1.undo(10);

    // The internal undo stack now looks like this:
    // 0. U - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. U - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")
    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "default".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 0);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 0);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[0].clone())]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    // Try undoing 1 step on session 2, should do nothing
    session1.undo(1);

    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "default".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 0);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 0);

    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(get_user_log_and_clear(), vec![]);
}

#[test]
fn test_redo_document_one_user() {
    // Both session are owned by the same user
    let (project, session1, session2, doc_uuid, transactions) = create_undo_redo_test_setup();
    let session_doc_closure = project.open_data::<TestModule>(doc_uuid).unwrap();
    let session_user_closure = project.open_data::<TestModule>(doc_uuid).unwrap();
    // closures for getting a current snapshot of both data sections and the internal log
    // Since all sessions are owned by the same user, the data should be the same
    let document = || session_doc_closure.snapshot().persistent;
    let user = || session_user_closure.snapshot().persistent_user;
    let get_doc_log_and_clear = || {
        let doc_log_uuid = document().logging_uuid;
        let doc_log = get_transaction_log(doc_log_uuid);
        clear_transaction_log(doc_log_uuid);
        doc_log
    };
    let get_user_log_and_clear = || {
        let user_log_uuid = user().logging_uuid;
        let user_log = get_transaction_log(user_log_uuid);
        clear_transaction_log(user_log_uuid);
        user_log
    };

    session1.undo(10);
    session2.undo(10);

    // The internal undo stack now looks like this:
    // (A is applied, U is undone, F is failed, Ar is applied but redone(undone+applied),
    // Fr is failed but redone(undone+applied failed))
    //
    // 0. U - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. U - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")
    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "default".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 0);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 0);

    // We already tested undo, so we can assume that this log is correct
    get_doc_log_and_clear();
    get_user_log_and_clear();

    // Should only redo as much as possible
    session1.redo(10);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. A - User(s1): SetWord("word_b")
    // 2. U - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. A - Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "word_b".to_string());
    assert_eq!(document().single_word, "word_e".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 3);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 0);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Applied, transactions[0].clone()),
            (TransactionStatus::Applied, transactions[4].clone()),
        ]
    );
    assert_eq!(
        get_user_log_and_clear(),
        vec![(TransactionStatus::Applied, transactions[1].clone())]
    );

    session2.redo(1);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. A - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. Ar- Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "word_b".to_string());
    assert_eq!(document().single_word, "word_e".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 3);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[4].clone()),
            (TransactionStatus::Applied, transactions[2].clone()),
            (TransactionStatus::Applied, transactions[4].clone())
        ]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    session1.undo(1);

    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. A - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "word_b".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 2);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[4].clone()),]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    // This should not change anything
    session2.redo(0);
    session1.redo(0);

    assert_eq!(user().single_word, "word_b".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 2);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(get_user_log_and_clear(), vec![]);

    session1.undo(2);
    // The internal undo stack now looks like this:
    // 0. U - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. Ar - Doc(s2): SetWord("word_c")
    // 3. U - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "default".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 0);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 1);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[2].clone()),
            (TransactionStatus::Undone, transactions[0].clone()),
            (TransactionStatus::Applied, transactions[2].clone()),
        ]
    );
    assert_eq!(
        get_user_log_and_clear(),
        vec![(TransactionStatus::Undone, transactions[1].clone()),]
    );

    session2.redo(1);
    // The internal undo stack now looks like this:
    // 0. U - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. A - Doc(s2): SetWord("word_c")
    // 3. A - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "word_d".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 0);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 2);

    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(
        get_user_log_and_clear(),
        vec![(TransactionStatus::Applied, transactions[3].clone()),]
    );

    session1.redo(1);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetWord("word_a")
    // 1. U - User(s1): SetWord("word_b")
    // 2. Ar- Doc(s2): SetWord("word_c")
    // 3. A - User(s2): SetWord("word_d")
    // 4. U - Doc(s1): SetWord("word_e")

    assert_eq!(user().single_word, "word_d".to_string());
    assert_eq!(document().single_word, "word_c".to_string());

    let loc = session1.undo_redo_list().1;
    assert_eq!(loc, 1);
    let loc = session2.undo_redo_list().1;
    assert_eq!(loc, 2);

    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[2].clone()),
            (TransactionStatus::Applied, transactions[0].clone()),
            (TransactionStatus::Applied, transactions[2].clone()),
        ]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);
}

#[test]
fn test_undo_redo_on_failed_transactions() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document();
    let data_uuid = project
        .open_document(doc_uuid)
        .unwrap()
        .create_data::<TestModule>();

    let mut session2 = project.open_data::<TestModule>(data_uuid).unwrap();
    let mut session3 = project.open_data::<TestModule>(data_uuid).unwrap();
    let mut session1 = project.open_data::<TestModule>(data_uuid).unwrap();

    let transactions = vec![
        (
            1,
            TransactionArgs::Persistent(TestTransaction::SetNumber(101)),
        ),
        (
            1,
            TransactionArgs::PersistentUser(TestTransaction::SetNumber(201)),
        ),
        (
            1,
            TransactionArgs::PersistentUser(TestTransaction::SetNumber(51)),
        ),
        (
            2,
            TransactionArgs::Persistent(TestTransaction::SetNumber(301)),
        ),
        (
            3,
            TransactionArgs::PersistentUser(TestTransaction::FailIfNumberIsOver100),
        ),
        (
            2,
            TransactionArgs::Persistent(TestTransaction::SetNumber(1)),
        ),
        (
            3,
            TransactionArgs::Persistent(TestTransaction::FailIfNumberIsOver100),
        ),
    ];

    // Apply transactions
    for (session_number, transaction) in &transactions {
        let session = match session_number {
            1 => &mut session1,
            2 => &mut session2,
            3 => &mut session3,
            _ => panic!("Invalid session number"),
        };
        assert!(session.apply(transaction.clone()).is_ok());
    }

    // Now we cast away unneeded information from the transactions list
    let transactions: Vec<TestTransaction> = transactions
        .into_iter()
        .map(|(_, transaction)| match transaction {
            TransactionArgs::Persistent(transaction) => transaction,
            TransactionArgs::PersistentUser(transaction) => transaction,
            TransactionArgs::Session(transaction) => transaction,
            TransactionArgs::Shared(transaction) => transaction,
        })
        .collect();

    // This will copy the internal logging id from the internal document
    // to our sessions. This is needed to inspect the applied undo/apply
    // transactions until we implement a non copy based distribution
    // of the new state.
    // TODO: Remove this when undo/redo doesn't copy
    session2.redo(1);

    // We already tested the undo_redo_list in the previous tests

    let session_doc_closure = project.open_data::<TestModule>(data_uuid).unwrap();
    let session_user_closure = project.open_data::<TestModule>(data_uuid).unwrap();
    // closures for getting a current snapshot of both data sections and the internal log
    // Since all sessions are owned by the same user, the data should be the same
    let document = || session_doc_closure.snapshot().persistent;
    let user = || session_user_closure.snapshot().persistent_user;
    let get_doc_log_and_clear = || {
        let doc_log_uuid = document().logging_uuid;
        let doc_log = get_transaction_log(doc_log_uuid);
        clear_transaction_log(doc_log_uuid);
        doc_log
    };
    let get_user_log_and_clear = || {
        let user_log_uuid = user().logging_uuid;
        let user_log = get_transaction_log(user_log_uuid);
        clear_transaction_log(user_log_uuid);
        user_log
    };

    // Now we test if the undo/redo works as expected on failed transactions

    // The internal undo stack now looks like this:
    // (A is applied, U is undone, F is failed, Ar is applied but redone(undone+applied),
    // Fr is failed but redone(undone+applied failed))
    //
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. A - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. A - User(s3): FailIfNumberIsOver100
    // 5. A - Doc(s2): SetNumber(1)
    // 6. A - Doc(s3): FailIfNumberIsOver100

    get_doc_log_and_clear();
    get_user_log_and_clear();

    session1.undo(1);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. U - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. Fr- User(s3): FailIfNumberIsOver100
    // 5. A - Doc(s2): SetNumber(1)
    // 6. A - Doc(s3): FailIfNumberIsOver100

    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(
        get_user_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[4].clone()),
            (TransactionStatus::Undone, transactions[2].clone()),
            (TransactionStatus::ApplyFailed, transactions[4].clone()),
        ]
    );

    session2.undo(2);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. U - User(s1): SetNumber(51)
    // 3. U - Doc(s2): SetNumber(301)
    // 4. F - User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. Fr- Doc(s3): FailIfNumberIsOver100
    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[6].clone()),
            (TransactionStatus::Undone, transactions[5].clone()),
            (TransactionStatus::Undone, transactions[3].clone()),
            (TransactionStatus::ApplyFailed, transactions[6].clone()),
        ]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    session2.redo(1);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. U - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. F - User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. Fr- Doc(s3): FailIfNumberIsOver100
    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Applied, transactions[3].clone()),
            (TransactionStatus::ApplyFailed, transactions[6].clone()),
        ]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    session1.undo(1);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. U - User(s1): SetNumber(201)
    // 2. U - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. Ar- User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. F - Doc(s3): FailIfNumberIsOver100
    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(
        get_user_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[1].clone()),
            (TransactionStatus::Applied, transactions[4].clone()),
        ]
    );

    session1.redo(2);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. A - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. Ar- User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. F - Doc(s3): FailIfNumberIsOver100
    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(
        get_user_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[4].clone()),
            (TransactionStatus::Applied, transactions[1].clone()),
            (TransactionStatus::Applied, transactions[2].clone()),
            (TransactionStatus::Applied, transactions[4].clone()),
        ]
    );

    session3.undo(1);
    // This should only mark the transaction as undone, but otherwise do nothing
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. A - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. A - User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. U - Doc(s3): FailIfNumberIsOver100
    assert_eq!(get_doc_log_and_clear(), vec![]);
    assert_eq!(get_user_log_and_clear(), vec![]);

    session3.redo(1);
    // This should try to apply the transaction again, but fail again
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. A - User(s1): SetNumber(51)
    // 3. A - Doc(s2): SetNumber(301)
    // 4. A - User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. F - Doc(s3): FailIfNumberIsOver100
    assert_eq!(
        get_doc_log_and_clear(),
        vec![(TransactionStatus::ApplyFailed, transactions[6].clone())]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);

    session2.undo(1);
    // The internal undo stack now looks like this:
    // 0. A - Doc(s1): SetNumber(101)
    // 1. A - User(s1): SetNumber(201)
    // 2. A - User(s1): SetNumber(51)
    // 3. U - Doc(s2): SetNumber(301)
    // 4. A - User(s3): FailIfNumberIsOver100
    // 5. U - Doc(s2): SetNumber(1)
    // 6. Fr- Doc(s3): FailIfNumberIsOver100
    assert_eq!(
        get_doc_log_and_clear(),
        vec![
            (TransactionStatus::Undone, transactions[3].clone()),
            (TransactionStatus::ApplyFailed, transactions[6].clone())
        ]
    );
    assert_eq!(get_user_log_and_clear(), vec![]);
}
