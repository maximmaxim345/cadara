mod common;
use common::{minimal_test_module::MinimalTestModule, test_module::TestModule};
use project::*;
use uuid::Uuid;

#[test]
fn test_attempt_open_data_with_incorrect_module() {
    let project = Project::new("Project".to_string()).create_session();

    let doc_uuid = project.create_document();
    let doc = project.open_document(doc_uuid).unwrap();

    let data_uuid = doc.create_data::<TestModule>();

    assert!(doc.open_data_by_uuid::<TestModule>(data_uuid).is_some());
    assert!(doc
        .open_data_by_uuid::<MinimalTestModule>(data_uuid)
        .is_none());
}

#[test]
fn test_open_nonexistent_data() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document();
    let doc = project.open_document(doc_uuid).unwrap();

    assert!(doc
        .open_data_by_uuid::<TestModule>(Uuid::new_v4())
        .is_none());
}

#[test]
fn test_open_data_by_type() {
    let project = Project::new("Project".to_string()).create_session();
    let doc_uuid = project.create_document();
    let doc = project.open_document(doc_uuid).unwrap();

    let _data1 = doc.create_data::<TestModule>();
    let _data2 = doc.create_data::<TestModule>();
    let _other_data = doc.create_data::<MinimalTestModule>();

    let data_sections: Vec<_> = doc.open_data_by_type::<TestModule>();
    assert_eq!(data_sections.len(), 2);
}

#[test]
fn test_open_data_in_wrong_document() {
    let project = Project::new("Project".to_string()).create_session();

    let doc1 = project.create_document();
    let doc1 = project.open_document(doc1).unwrap();
    let data_in_doc1 = doc1.create_data::<TestModule>();

    let doc2 = project.create_document();
    let doc2 = project.open_document(doc2).unwrap();

    assert!(doc1.open_data_by_uuid::<TestModule>(data_in_doc1).is_some());
    assert!(doc2.open_data_by_uuid::<TestModule>(data_in_doc1).is_none());
}
