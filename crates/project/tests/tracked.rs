mod common;
use common::{minimal_test_module::MinimalTestModule, setup_project};
use project::*;
use std::sync::Arc;

#[test]
fn test_tracked_view_not_accessed() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();

    let mut project = Project::new();
    {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();
        let _ = view.create_data::<MinimalTestModule>(&mut cb);
        project.apply_changes(cb, &reg).unwrap();
    }

    let view = Arc::new(project.create_view(&reg).unwrap());
    let (_tracked_view, recorder) = TrackedProjectView::new(view);

    let validator = recorder.freeze();
    assert!(!validator.was_accessed());
}

#[test]
fn test_tracked_view_basic_access() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();

    let mut project = Project::new();
    let data_id;
    {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();
        data_id = view.create_data::<MinimalTestModule>(&mut cb);
        project.apply_changes(cb, &reg).unwrap();
    }

    let view = Arc::new(project.create_view(&reg).unwrap());
    let (tracked_view, recorder) = TrackedProjectView::new(view);

    // Access data to generate tracking events
    let _data = tracked_view
        .open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap();

    let validator = recorder.freeze();
    assert!(validator.was_accessed());
}

#[test]
fn test_document_operations() {
    let p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Test document creation
    let mut cb = ChangeBuilder::from(&p.project);
    let _new_doc =
        tracked_view.create_document(&mut cb, Path::new("/new_doc".to_string()).unwrap());

    // Test document access and data operations within document
    let doc = tracked_view.open_document(p.doc1).unwrap();
    let data_by_type: Vec<_> = doc.open_data_by_type::<MinimalTestModule>().collect();
    assert!(!data_by_type.is_empty());

    // Test document data access by ID
    let data = doc
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();
    assert_eq!(data.session_data().num, 10);

    let validator = recorder.freeze();
    assert!(validator.was_accessed());
}

#[test]
fn test_data_operations() {
    let p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Test different ways to access data
    let by_id = tracked_view.open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data);
    assert!(by_id.is_some());

    let by_type: Vec<_> = tracked_view
        .open_data_by_type::<MinimalTestModule>()
        .collect();
    assert!(!by_type.is_empty());

    let validator = recorder.freeze();
    assert!(validator.was_accessed());
}

#[test]
fn test_different_project_validation() {
    let p1 = setup_project();
    let p2 = setup_project();

    let view1 = Arc::new(p1.view());
    let view2 = Arc::new(p2.view());

    let (tracked_view, recorder) = TrackedProjectView::new(view1.clone());

    // Generate some access events
    let _doc = tracked_view.open_document(p1.doc1);

    let validator = recorder.freeze();

    // Should be invalid for different projects
    assert!(!validator.is_cache_valid(&view1, &view2));
}

#[test]
fn test_cache_validation_open_data_by_type() {
    let mut p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Access data by type to generate tracking event
    let data_list: Vec<_> = tracked_view
        .open_data_by_type::<MinimalTestModule>()
        .collect();
    assert!(!data_list.is_empty());

    let validator = recorder.freeze();
    assert!(validator.is_cache_valid(&view, &view));

    // Create new data of same type
    let mut cb = ChangeBuilder::from(&view);
    tracked_view.create_data::<MinimalTestModule>(&mut cb);
    p.project.apply_changes(cb, &p.reg).unwrap();
    let new_view = p.view();

    // Cache should be invalid as the list of data of this type changed
    assert!(!validator.is_cache_valid(&view, &new_view));
}

#[test]
fn test_cache_validation_document_data_by_id() {
    let mut p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Access document data by ID
    let doc = tracked_view.open_document(p.doc1).unwrap();
    let data = doc
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();
    let _session = data.session_data(); // Access some data to ensure it's tracked

    let validator = recorder.freeze();
    assert!(validator.is_cache_valid(&view, &view));

    // Move data to different document
    let mut cb = ChangeBuilder::from(&view);
    let mut new_doc =
        tracked_view.create_document(&mut cb, Path::new("/new_doc".to_string()).unwrap());
    data.move_to_planned_document(&mut new_doc);
    p.project.apply_changes(cb, &p.reg).unwrap();
    let new_view = p.view();

    // Cache should be invalid as the data moved documents
    assert!(!validator.is_cache_valid(&view, &new_view));
}

#[test]
fn test_cache_validation_document_data_by_type() {
    let mut p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Access document data by type
    let doc = tracked_view.open_document(p.doc1).unwrap();
    let data_list: Vec<_> = doc.open_data_by_type::<MinimalTestModule>().collect();
    assert!(!data_list.is_empty());

    let validator = recorder.freeze();
    assert!(validator.is_cache_valid(&view, &view));

    // Add new data of same type to document
    let mut cb = ChangeBuilder::from(&view);
    let _data = doc.create_data::<MinimalTestModule>(&mut cb);
    p.project.apply_changes(cb, &p.reg).unwrap();
    let new_view = p.view();

    // Cache should be invalid as document's data list changed
    assert!(!validator.is_cache_valid(&view, &new_view));
}

#[test]
fn test_cache_validation_complex_scenario() {
    let mut p = setup_project();
    let view = Arc::new(p.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Generate multiple types of access events
    let doc = tracked_view.open_document(p.doc1).unwrap();

    // Access by type globally
    let _global_data: Vec<_> = tracked_view
        .open_data_by_type::<MinimalTestModule>()
        .collect();

    // Access by type in document
    let _doc_data: Vec<_> = doc.open_data_by_type::<MinimalTestModule>().collect();

    // Access specific data
    let data = doc
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();
    let _session = data.session_data();

    let validator = recorder.freeze();
    assert!(validator.is_cache_valid(&view, &view));

    // Make various modifications
    let mut cb = ChangeBuilder::from(&view);

    // Create new data
    let _new_data_id = tracked_view.create_data::<MinimalTestModule>(&mut cb);

    // Move existing data
    let mut new_doc =
        tracked_view.create_document(&mut cb, Path::new("/new_doc".to_string()).unwrap());
    data.move_to_planned_document(&mut new_doc);

    // Modify data content
    data.apply_session(42, &mut cb);

    p.project.apply_changes(cb, &p.reg).unwrap();
    let new_view = p.view();

    // Cache should be invalid due to multiple changes
    assert!(!validator.is_cache_valid(&view, &new_view));
}

#[test]
fn test_cache_validation_edge_cases() {
    // Create two separate projects to get valid but non-existent IDs
    let mut p1 = setup_project();
    let p2 = setup_project();

    let view = Arc::new(p1.view());
    let (tracked_view, recorder) = TrackedProjectView::new(view.clone());

    // Test with document that exists in p2 but not in p1
    let _missing_doc = tracked_view.open_document(p2.doc1);

    // Test with non-existent data (using data ID from p2)
    let doc = tracked_view.open_document(p1.doc1).unwrap();
    let _missing_data = doc.open_data_by_id::<MinimalTestModule>(p2.doc1_minimal_data);

    // Test with empty data type list
    let _empty_list = doc.open_data_by_type::<MinimalTestModule>();

    let validator = recorder.freeze();

    // Modify project
    let mut cb = ChangeBuilder::from(&view);
    let _data = doc.create_data::<MinimalTestModule>(&mut cb);
    p1.project.apply_changes(cb, &p1.reg).unwrap();
    let new_view = p1.view();

    // Validate cache with edge cases
    assert!(!validator.is_cache_valid(&view, &new_view));
}
