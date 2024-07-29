mod common;
use project::*;
use uuid::Uuid;

#[test]
fn test_attempt_open_nonexistent_document() {
    let project = Project::new("Project".to_string()).create_session();

    let doc = project.open_document(Uuid::new_v4());
    assert!(doc.is_none());
}

#[test]
fn test_open_document() {
    let project = Project::new("Project".to_string()).create_session();

    let doc_uuid = project.create_document();

    let doc = project.open_document(doc_uuid);
    assert!(doc.is_some());
}
