mod common;

use common::test_module::*;

use project::document::transaction::TransactionArgs;
use project::*;
use serde::de::DeserializeSeed;
use utils::Transaction;

#[test]
fn test_serde_project_json() {
    let doc_uuid;
    let json;

    {
        let project = Project::new("Project".to_string());
        doc_uuid = project.create_document::<TestModule>();

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

        // First serialize, then close sessions
        json = serde_json::to_string_pretty(&project).unwrap();
        println!("{}", json);
        // return;
    }

    {
        let seed = ProjectSeed {
            registry: &{
                let mut registry = ModuleRegistry::default();
                registry.register::<TestModule>();
                registry
            },
        };

        let deserializer = &mut serde_json::Deserializer::from_str(&json);
        let project: Project = seed.deserialize(deserializer).unwrap();

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
