mod common;
use common::test_module::*;

use project::data::transaction::TransactionArgs;
use project::*;

#[test]
fn test_document_persistent_data() {
    let project = Project::new("Project".to_string()).create_view();

    let doc = project.create_document();
    let doc = project.open_document(doc).unwrap();
    let data_uuid = doc.create_data::<TestModule>();
    {
        let mut data = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());

        assert!(data
            .apply(TransactionArgs::Persistent(transaction.clone()))
            .is_ok());
        assert!(data
            .apply(TransactionArgs::PersistentUser(transaction.clone()))
            .is_ok());
        assert!(data
            .apply(TransactionArgs::Session(transaction.clone()))
            .is_ok());
        assert!(data.apply(TransactionArgs::Shared(transaction)).is_ok());
    }
    {
        let data = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let snapshot = data.snapshot();

        assert_eq!(
            snapshot.persistent.single_word, "Test",
            "Persistent data should be shared"
        );
        assert_eq!(
            snapshot.persistent_user.single_word, "Test",
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
    let project = Project::new("Project".to_string()).create_view();
    let doc = project.create_document();
    let doc = project.open_document(doc).unwrap();
    let data_uuid = doc.create_data::<TestModule>();
    let mut session1 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
    let session2 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
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
        let session3 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let snapshot = session3.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "Test",
            "Shared state should be shared with new sessions",
        );
    }
}

#[test]
fn test_reset_of_shared_state() {
    let project = Project::new("Project".to_string()).create_view();
    let doc = project.create_document();
    let doc = project.open_document(doc).unwrap();
    let data_uuid = doc.create_data::<TestModule>();
    {
        let mut session1 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());
        assert!(session1.apply(TransactionArgs::Shared(transaction)).is_ok());
        let snapshot = session1.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "Test",
            "Shared state should be shared"
        );
    }
    {
        let data = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let snapshot = data.snapshot();
        assert_eq!(
            snapshot.shared.single_word, "default",
            "Shared state should be reset after closing all sessions"
        );
    }
}

#[test]
fn test_user_state() {
    let project = Project::new("Project".to_string()).create_view();
    let doc = project.create_document();
    let doc = project.open_document(doc).unwrap();
    let data_uuid = doc.create_data::<TestModule>();
    {
        let mut session1 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let transaction = TestTransaction::SetWord("Test".to_string());
        assert!(session1
            .apply(TransactionArgs::Session(transaction))
            .is_ok());
        let session2 = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let snapshot = session2.snapshot();
        assert_eq!(
            snapshot.session.single_word, "default",
            "User state should not be shared"
        );
    }
    {
        let data = doc.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
        let snapshot = data.snapshot();
        assert_eq!(
            snapshot.session.single_word, "default",
            "User state should be reset after closing all sessions"
        );
    }
}
