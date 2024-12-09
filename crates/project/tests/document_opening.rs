mod common;
use document::DocumentUuid;
use project::*;

#[test]
fn test_attempt_open_nonexistent_document() {
    let project = Project::new("Project".to_string()).create_view();

    let doc = project.open_document(DocumentUuid::new_v4());
    assert!(doc.is_none());
}

#[test]
fn test_open_document() {
    let project = Project::new("Project".to_string()).create_view();

    let doc_uuid = project.create_document();

    let doc = project.open_document(doc_uuid);
    assert!(doc.is_some());
}
