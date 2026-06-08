mod common;

use common::*;
use project::{ChangeBuilder, ModuleRegistry, Project};

fn setup() -> (Project, project::DataId, ModuleRegistry) {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();
    let mut project = Project::new();

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    let doc = *view.create_document(&mut cb, "/d".try_into().unwrap());
    project.apply_changes(cb, &reg).unwrap();

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    let data_id = *view
        .open_document(doc)
        .unwrap()
        .create_data::<MinimalTestModule>(&mut cb);
    project.apply_changes(cb, &reg).unwrap();

    (project, data_id, reg)
}

fn read(p: &Project, reg: &ModuleRegistry, data_id: project::DataId) -> i32 {
    p.create_view(reg)
        .unwrap()
        .open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .persistent
        .num
}

fn edit(p: &mut Project, reg: &ModuleRegistry, data_id: project::DataId, val: i32) {
    let mut cb = ChangeBuilder::from(p);
    let view = p.create_view(reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(val, &mut cb);
    p.apply_changes(cb, reg).unwrap();
}

#[test]
fn three_replicas_in_cycle_all_converge() {
    let (mut a, data_id, reg) = setup();
    let mut b = a.clone_for_test_replica();
    let mut c = a.clone_for_test_replica();

    edit(&mut a, &reg, data_id, 1);
    edit(&mut b, &reg, data_id, 2);
    edit(&mut c, &reg, data_id, 3);

    a.merge_remote(&b, &reg).unwrap();
    b.merge_remote(&c, &reg).unwrap();
    c.merge_remote(&a, &reg).unwrap();
    a.merge_remote(&c, &reg).unwrap();
    b.merge_remote(&a, &reg).unwrap();

    let va = read(&a, &reg, data_id);
    let vb = read(&b, &reg, data_id);
    let vc = read(&c, &reg, data_id);
    assert_eq!(va, vb);
    assert_eq!(vb, vc);
}

#[test]
fn concurrent_edits_lww_by_lamport_session() {
    let (mut a, data_id, reg) = setup();

    // Reset a's session so both replicas mint sessions concurrently;
    // otherwise b's mandatory NewSession bumps its edit's lamport above a's,
    // and the (lamport, session_id) tiebreaker never engages.
    a.reset_session();
    let mut b = a.clone_for_test_replica();

    edit(&mut a, &reg, data_id, 100);
    edit(&mut b, &reg, data_id, 200);

    let a_session = a.current_session().expect("a has a session after edit");
    let b_session = b.current_session().expect("b has a session after edit");

    a.merge_remote(&b, &reg).unwrap();
    b.merge_remote(&a, &reg).unwrap();

    let va = read(&a, &reg, data_id);
    let vb = read(&b, &reg, data_id);
    assert_eq!(va, vb, "both replicas converge");
    assert!(
        va == 100 || va == 200,
        "converged value must be one of the two edits"
    );

    let expected = if a_session > b_session { 100 } else { 200 };
    assert_eq!(
        va, expected,
        "LWW winner determined by (lamport, session_id)"
    );
}

#[test]
fn session_a_undo_does_not_affect_session_b() {
    let (mut a, data_id, reg) = setup();
    let mut b = a.clone_for_test_replica();

    edit(&mut a, &reg, data_id, 5);
    edit(&mut b, &reg, data_id, 9);

    let mut cb = ChangeBuilder::from(&a);
    cb.undo();
    a.apply_changes(cb, &reg).unwrap();
    assert_eq!(
        read(&a, &reg, data_id),
        0,
        "A's undo should revert its own edit"
    );

    a.merge_remote(&b, &reg).unwrap();
    b.merge_remote(&a, &reg).unwrap();

    // Both sides see B's value (A's was undone).
    assert_eq!(read(&a, &reg, data_id), 9);
    assert_eq!(read(&b, &reg, data_id), 9);
}

#[test]
fn merge_different_project_rejected() {
    let (mut a, _, reg) = setup();
    let unrelated = Project::new();
    let err = a.merge_remote(&unrelated, &reg).unwrap_err();
    assert!(matches!(err, project::MergeError::DifferentProject));
}
