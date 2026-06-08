mod common;

use common::*;
use project::{BranchId, ChangeBuilder, ModuleRegistry, Project};

fn fresh_project() -> (Project, ModuleRegistry) {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();
    (Project::new(), reg)
}

fn read_num(
    project: &Project,
    reg: &ModuleRegistry,
    branch: BranchId,
    data_id: project::DataId,
) -> i32 {
    project
        .create_view_at(reg, branch, u64::MAX)
        .unwrap()
        .open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .persistent
        .num
}

#[test]
fn edits_on_unmerged_branch_invisible_from_main() {
    let (mut project, reg) = fresh_project();

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

    let main = project.current_branch();
    let feature = BranchId::new();

    // Fork: give feature visibility of main's current state.
    project.merge_branch(main, feature);

    project.switch_branch(feature);
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(42, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    assert_eq!(read_num(&project, &reg, main, data_id), 0);
    assert_eq!(read_num(&project, &reg, feature, data_id), 42);
}

#[test]
fn merge_branch_imports_snapshot() {
    let (mut project, reg) = fresh_project();
    let main = project.current_branch();
    let feature = BranchId::new();

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

    // Fork: give feature visibility of main's current state.
    project.merge_branch(main, feature);

    project.switch_branch(feature);
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(7, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    project.switch_branch(main);
    project.merge_branch(feature, main);

    assert_eq!(read_num(&project, &reg, main, data_id), 7);
}

#[test]
fn edits_after_merge_not_visible_until_re_merge() {
    let (mut project, reg) = fresh_project();
    let main = project.current_branch();
    let feature = BranchId::new();

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

    // Fork: give feature visibility of main's current state.
    project.merge_branch(main, feature);

    project.switch_branch(feature);
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(5, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    project.switch_branch(main);
    project.merge_branch(feature, main);

    project.switch_branch(feature);
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(11, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    assert_eq!(read_num(&project, &reg, main, data_id), 5);

    project.switch_branch(main);
    project.merge_branch(feature, main);
    assert_eq!(read_num(&project, &reg, main, data_id), 11);
}

#[test]
fn undo_of_merge_branch_hides_imported_ops() {
    let (mut project, reg) = fresh_project();
    let main = project.current_branch();
    let feature = BranchId::new();

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

    // Fork: give feature visibility of main's current state.
    project.merge_branch(main, feature);

    project.switch_branch(feature);
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(9, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    // Reset session so undo targets only the merge, not the CreateDocument/CreateData setup.
    project.switch_branch(main);
    project.reset_session();
    project.merge_branch(feature, main);
    assert_eq!(read_num(&project, &reg, main, data_id), 9);

    project.undo();
    assert_eq!(read_num(&project, &reg, main, data_id), 0);

    project.redo();
    assert_eq!(read_num(&project, &reg, main, data_id), 9);
}

#[test]
fn multi_hop_merge_respects_per_hop_snapshot_order() {
    let (mut project, reg) = fresh_project();
    let main = project.current_branch();
    let feature = BranchId::new();
    let hot = BranchId::new();

    // Setup: one document with one data on main.
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

    // Bring main's setup into feature and hot so each branch has the data.
    project.switch_branch(feature);
    project.merge_branch(main, feature);

    project.switch_branch(hot);
    project.merge_branch(main, hot);

    // hot edits the data.
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(42, &mut cb);
    project.apply_changes(cb, &reg).unwrap();

    // Merge feature -> main FIRST (feature has no edits from hot yet).
    project.switch_branch(main);
    project.merge_branch(feature, main);

    // Then merge hot -> feature (introducing hot's edit into feature).
    project.switch_branch(feature);
    project.merge_branch(hot, feature);

    // Main should still see the default value: hot's edit got into feature
    // AFTER feature was merged into main, so main never absorbed it.
    assert_eq!(read_num(&project, &reg, main, data_id), 0);

    // Feature should see hot's edit, since hot was merged into feature last.
    assert_eq!(read_num(&project, &reg, feature, data_id), 42);
}
