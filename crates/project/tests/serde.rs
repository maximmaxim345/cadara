mod common;
use common::{minimal_test_module::MinimalTestModule, setup_project};
use serde::de::DeserializeSeed;

use project::*;

#[test]
fn test_serialize_deserialize_json() {
    let p = setup_project();

    let json = serde_json::to_string_pretty(&p.project).unwrap();

    let seed = ProjectDeserializer { registry: &p.reg };

    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    let loaded_project = seed.deserialize(deserializer).unwrap();

    let view = p.view();
    let loaded_view = loaded_project.create_view(&p.reg).unwrap();

    let docview = view.open_document(p.doc1).unwrap();
    let dview = docview
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();

    let loaded_docview = loaded_view.open_document(p.doc1).unwrap();
    let loaded_dview = loaded_docview
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();

    assert_eq!(
        dview.persistent, loaded_dview.persistent,
        "Persistent data should be serialized"
    );
    assert_eq!(
        dview.persistent_user, loaded_dview.persistent_user,
        "Persistent User data should be serialized"
    );
    assert_eq!(
        loaded_dview.session_data.num, 0,
        "Session data should not be serialized"
    );
    assert_eq!(
        loaded_dview.shared_data.num, 0,
        "Session data should not be serialized"
    );
}

#[test]
fn test_deserialize_unknown_module_json() {
    let p = setup_project();

    let json = serde_json::to_string_pretty(&p.project).unwrap();

    let empty_reg = ModuleRegistry::new();
    let seed = ProjectDeserializer {
        registry: &empty_reg,
    };

    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    assert!(seed.deserialize(deserializer).is_err());
}
