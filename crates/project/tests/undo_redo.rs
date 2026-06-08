mod common;

use common::*;
use project::{ChangeBuilder, ModuleRegistry, Project};

/// Build a project with one document, one MinimalTestModule data, and apply a
/// sequence of integer-setting transactions to its persistent data, with the
/// special markers 0 (Undo) and -1 (Redo).
fn run_sequence(seq: &[i32]) -> i32 {
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

    // Start a fresh session so that undo/redo in the sequence below cannot
    // target the structural setup changes (CreateDocument, CreateData).
    project.reset_session();

    for &v in seq {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();
        match v {
            0 => cb.undo(),
            -1 => cb.redo(),
            n => view
                .open_data_by_id::<MinimalTestModule>(data_id)
                .unwrap()
                .apply_persistent(n, &mut cb),
        }
        project.apply_changes(cb, &reg).unwrap();
    }

    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .persistent
        .num
}

#[test]
fn trace_simple_undo_after_two_edits() {
    // [x6=6, x7=7, x8=8, Undo, x15=15] -> active includes x6, x7, x15 (x8 undone).
    // LWW per-field: lamport-ordered apply gives final num = 15.
    assert_eq!(run_sequence(&[6, 7, 8, 0, 15]), 15);
}

#[test]
fn trace_three_undos_after_branchy_history() {
    // [x6, x7, x8, Undo, x15, Undo, Undo] -> active = {x6}
    assert_eq!(run_sequence(&[6, 7, 8, 0, 15, 0, 0]), 6);
}

#[test]
fn trace_redo_after_new_edit_brings_back_undone() {
    // [x6, Undo, x7, Redo] -> active {x6, x7}; LWW winner by lamport is x7.
    assert_eq!(run_sequence(&[6, 0, 7, -1]), 7);
}

#[test]
fn trace_undo_after_redo_targets_max_active() {
    // [x6, Undo, x7, Redo, Undo] -> active {x6}; state [x6].
    assert_eq!(run_sequence(&[6, 0, 7, -1, 0]), 6);
}

#[test]
fn undo_on_empty_active_is_noop() {
    // Just an Undo on a fresh data; no edits to cancel. Expect default (0).
    assert_eq!(run_sequence(&[0]), 0);
}

#[test]
fn redo_on_empty_buffer_is_noop() {
    assert_eq!(run_sequence(&[5, -1]), 5);
}
