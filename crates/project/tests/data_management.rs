mod common;
use common::{minimal_test_module::MinimalTestModule, setup_project, test_module::*};

use project::*;

#[test]
fn test_open_data_by_id() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();

    let mut project = Project::new();
    let data_id;
    {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();

        data_id = view.create_data::<MinimalTestModule>(&mut cb);
        project.apply_changes(cb, &reg).unwrap();
    };
    let v = project.create_view(&reg).unwrap();
    assert!(v.open_data_by_id::<MinimalTestModule>(data_id).is_some());
}

#[test]
fn test_open_data_by_type() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();
    reg.register::<TestModule>();

    let mut project = Project::new();
    {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();

        view.create_data::<MinimalTestModule>(&mut cb);
        view.create_data::<MinimalTestModule>(&mut cb);
        view.create_data::<TestModule>(&mut cb);
        project.apply_changes(cb, &reg).unwrap();
    };
    let v = project.create_view(&reg).unwrap();
    assert_eq!(v.open_data_by_type::<MinimalTestModule>().count(), 2);
    assert_eq!(v.open_data_by_type::<TestModule>().count(), 1);
}

#[test]
fn test_open_data_wrong_module() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();

    let mut project = Project::new();
    let data_id;
    {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();

        data_id = view.create_data::<MinimalTestModule>(&mut cb);
        project.apply_changes(cb, &reg).unwrap();
    };
    let v = project.create_view(&reg).unwrap();
    assert!(v.open_data_by_id::<TestModule>(data_id).is_none());
}

#[test]
fn test_open_data_in_document() {
    let p = setup_project();
    let v = p.view();

    let doc1 = v.open_document(p.doc1).unwrap();
    assert!(doc1
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_some());
    assert_eq!(doc1.open_data_by_type::<MinimalTestModule>().count(), 1);
}

#[test]
fn test_open_data_in_wrong_document() {
    let p = setup_project();
    let v = p.view();

    let doc2 = v.open_document(p.doc2).unwrap();
    assert!(doc2
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_none());
    assert_eq!(doc2.open_data_by_type::<MinimalTestModule>().count(), 0);
}

#[test]
fn test_delete_document() {
    let mut p = setup_project();
    let v = p.view();

    let mut cb = ChangeBuilder::from(&v);
    v.open_document(p.doc1).unwrap().delete(&mut cb);

    p.project.apply_changes(cb, &p.reg).unwrap();
    let v = p.view();

    // Only doc1 and its data should be affected
    assert!(v.open_document(p.doc1).is_none());
    assert!(v
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_none());
    assert!(v.open_document(p.doc2).is_some());
    assert!(v.open_data_by_id::<TestModule>(p.doc2_test_data).is_some());
    assert!(v
        .open_data_by_id::<MinimalTestModule>(p.orphan_minimal_data)
        .is_some());
}

#[test]
fn test_delete_data() {
    let mut p = setup_project();
    let v = p.view();

    let minimal_data_count = v.open_data_by_type::<MinimalTestModule>().count();
    let test_data_count = v.open_data_by_type::<TestModule>().count();

    let mut cb = ChangeBuilder::from(&v);
    v.open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap()
        .delete(&mut cb);
    p.project.apply_changes(cb, &p.reg).unwrap();

    let v = p.view();

    assert!(v
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_none());

    let minimal_data_count_after = v.open_data_by_type::<MinimalTestModule>().count();
    let test_data_count_after = v.open_data_by_type::<TestModule>().count();
    assert_eq!(minimal_data_count - 1, minimal_data_count_after);
    assert_eq!(test_data_count, test_data_count_after);
}

#[test]
fn test_move_data_between_documents() {
    let mut p = setup_project();
    let v = p.view();
    let mut cb = ChangeBuilder::from(&v);

    let doc1 = v.open_document(p.doc1).unwrap();
    let doc2_test_data = v.open_data_by_id::<TestModule>(p.doc2_test_data).unwrap();
    doc2_test_data.move_to_document(&doc1, &mut cb);

    p.project.apply_changes(cb, &p.reg).unwrap();
    let v = p.view();

    let doc1 = v.open_document(p.doc1).unwrap();
    let doc2 = v.open_document(p.doc2).unwrap();

    assert!(doc2
        .open_data_by_id::<TestModule>(p.doc2_test_data)
        .is_none());
    assert!(doc1
        .open_data_by_id::<TestModule>(p.doc2_test_data)
        .is_some());
}

#[test]
fn test_make_orphan() {
    let mut p = setup_project();
    let v = p.view();

    let doc1_minimal_data = v
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();
    let mut cb = ChangeBuilder::from(&doc1_minimal_data);
    doc1_minimal_data.make_orphan(&mut cb);

    p.project.apply_changes(cb, &p.reg).unwrap();
    let v = p.view();

    let doc1 = v.open_document(p.doc1).unwrap();
    assert!(doc1
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_none());

    assert!(v
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .is_some());
}

#[test]
fn test_pending_document_data() {
    let mut p = setup_project();
    let v = p.view();
    let mut cb = ChangeBuilder::from(&v);

    let doc = v.create_document(&mut cb, "/new document".try_into().unwrap());
    let data = doc.create_data::<MinimalTestModule>(&mut cb);
    data.apply_persistent(11, &mut cb);
    data.apply_persistent_user(12, &mut cb);
    data.apply_session(13, &mut cb);
    data.apply_shared(14, &mut cb);

    p.project.apply_changes(cb, &p.reg).unwrap();
    let v = p.view();

    let doc = v.open_document(*doc).unwrap();
    let data = doc.open_data_by_id::<MinimalTestModule>(*data).unwrap();

    assert_eq!(data.persistent.num, 11);
    assert_eq!(data.persistent_user.num, 12);
    assert_eq!(data.session_data.num, 13);
    assert_eq!(data.shared_data.num, 14);
}
