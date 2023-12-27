mod common;
use common::minimal_test_module::*;
use common::test_module::*;
use project::*;
use uuid::Uuid;

#[test]
fn test_attempt_open_nonexistent_document() {
    let project = Project::new("Project".to_string());

    let doc = project.open_document::<TestModule>(Uuid::new_v4());
    assert!(doc.is_none());
}

#[test]
fn test_open_document() {
    let project = Project::new("Project".to_string());

    let doc_uuid = project.create_document::<TestModule>();

    let doc = project.open_document::<TestModule>(doc_uuid);
    assert!(doc.is_some());
}

#[test]
fn test_attempt_open_document_with_incorrect_module() {
    let project = Project::new("Project".to_string());

    let doc_uuid = project.create_document::<MinimalTestModule>();

    let doc = project.open_document::<TestModule>(doc_uuid);
    assert!(doc.is_none());
}
