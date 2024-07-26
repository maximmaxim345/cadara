mod common;
use common::test_module::*;

use project::data::transaction::TransactionArgs;
use project::*;
use utils::Transaction;

#[test]
fn test_failed_transaction() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document::<TestModule>();
    {
        let mut session1 = project.open_document::<TestModule>(doc_uuid).unwrap();
        let session2 = project.open_document::<TestModule>(doc_uuid).unwrap();

        // Backup the state of the sessions and the document before applying the transactions
        let session1_snapshot_pre = session1.snapshot();
        let session2_snapshot_pre = session2.snapshot();
        let tmp_session_snapshot_pre = project
            .open_document::<TestModule>(doc_uuid)
            .unwrap()
            .snapshot();

        {
            // Apply 4 invalid transactions for each data section
            let transaction = TestTransaction::SetWord("Test Test".to_string());
            assert!(session1
                .apply(TransactionArgs::Document(transaction.clone()))
                .is_err());
            assert!(session1
                .apply(TransactionArgs::User(transaction.clone()))
                .is_err());
            assert!(session1
                .apply(TransactionArgs::Session(transaction.clone()))
                .is_err());
            assert!(session1
                .apply(TransactionArgs::Shared(transaction))
                .is_err());
        }

        // Verify that all the states are the same as before
        let session1_snapshot_post = session1.snapshot();
        let session2_snapshot_post = session2.snapshot();
        let tmp_session_snapshot_post = project
            .open_document::<TestModule>(doc_uuid)
            .unwrap()
            .snapshot();

        assert_eq!(
            session1_snapshot_pre, session1_snapshot_post,
            "Session 1 state should not have changed"
        );
        assert_eq!(
            session2_snapshot_pre, session2_snapshot_post,
            "Session 2 state should not have changed"
        );
        assert_eq!(
            tmp_session_snapshot_pre, tmp_session_snapshot_post,
            "Document state should not have changed"
        );
    }
    {
        let doc = project.open_document::<TestModule>(doc_uuid).unwrap();
        assert_eq!(
            doc.snapshot().session.odd_number,
            1,
            "User state should be reset after closing all sessions"
        );
    }
}
