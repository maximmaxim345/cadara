mod common;

use common::test_module::*;

use project::data::transaction::TransactionArgs;
use project::*;
use serde::de::DeserializeSeed;

#[test]
fn test_serde_project_json() {
    let doc_uuid;
    let data_uuid;
    let json;

    {
        let project = Project::new("Project".to_string()).create_session();
        doc_uuid = project.create_document();
        let doc = project.open_document(doc_uuid).unwrap();
        data_uuid = doc.create_data::<TestModule>();

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

        // First serialize, then close sessions
        json = serde_json::to_string_pretty(&project).unwrap();
        println!("{}", json);
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
        let project: ProjectView = seed.deserialize(deserializer).unwrap().create_session();

        let document = project.open_document(doc_uuid).unwrap();
        let data = document.open_data_by_uuid::<TestModule>(data_uuid).unwrap();
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
