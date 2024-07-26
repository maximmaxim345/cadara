mod common;
use common::test_module::*;

use project::data::transaction::TransactionArgs;
use project::*;
use utils::Transaction;

#[test]
fn test_document_persistent_data() {
    let project = Project::new("Project".to_string()).create_session();

    let doc_uuid = project.create_document::<TestModule>();
    {
        let mut doc = project.open_document::<TestModule>(doc_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());

        assert!(doc
            .apply(TransactionArgs::Document(transaction.clone()))
            .is_ok());
        assert!(doc
            .apply(TransactionArgs::User(transaction.clone()))
            .is_ok());
        assert!(doc
            .apply(TransactionArgs::Session(transaction.clone()))
            .is_ok());
        assert!(doc.apply(TransactionArgs::Shared(transaction)).is_ok());
    }
    {
        let doc = project.open_document::<TestModule>(doc_uuid).unwrap();
        let snapshot = doc.snapshot();

        assert_eq!(
            snapshot.document.single_word, "Test",
            "Document data should be shared"
        );
        assert_eq!(
            snapshot.user.single_word, "Test",
            "User data should be shared"
        );
        assert_eq!(
            snapshot.shared.single_word, "default",
            "User state should not be shared"
        );
    }
}

#[test]
fn test_shared_state() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document::<TestModule>();
    let mut session1 = project.open_document::<TestModule>(doc_uuid).unwrap();
    let session2 = project.open_document::<TestModule>(doc_uuid).unwrap();
    {
        let transaction = TestTransaction::SetWord("Test".to_string());
        assert!(session1.apply(TransactionArgs::Shared(transaction)).is_ok());
    }
    {
        let snapshot = session2.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "Test",
            "Shared state should be applied to all already open sessions"
        );
    }
    {
        let session3 = project.open_document::<TestModule>(doc_uuid).unwrap();
        let snapshot = session3.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "Test",
            "Shared state should be shared with new sessions",
        );
    }
}

#[test]
fn test_reset_of_shared_state() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document::<TestModule>();
    {
        let mut session1 = project.open_document::<TestModule>(doc_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());
        assert!(session1.apply(TransactionArgs::Shared(transaction)).is_ok());
        let snapshot = session1.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "Test",
            "Shared state should be shared"
        );
    }
    {
        let doc = project.open_document::<TestModule>(doc_uuid).unwrap();
        let snapshot = doc.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "default",
            "Shared state should be reset after closing all sessions"
        );
    }
}

#[test]
fn test_user_state() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document::<TestModule>();
    {
        let mut session1 = project.open_document::<TestModule>(doc_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());
        assert!(session1
            .apply(TransactionArgs::Session(transaction))
            .is_ok());
        let session2 = project.open_document::<TestModule>(doc_uuid).unwrap();
        let snapshot = session2.snapshot();
        assert_eq!(
            snapshot.session.single_word, "default",
            "User state should not be shared"
        );
    }
    {
        let doc = project.open_document::<TestModule>(doc_uuid).unwrap();
        let snapshot = doc.snapshot();
        assert_eq!(
            snapshot.session.single_word, "default",
            "User state should be reset after closing all sessions"
        );
    }
}
