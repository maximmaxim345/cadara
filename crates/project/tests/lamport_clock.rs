mod common;

use common::*;
use project::{ChangeBuilder, Project};

/// Reach into the serialized log to inspect lamport values.
///
/// Round-trips the project through JSON to access the log shape
/// without exposing internal fields.
fn lamports_of(project: &Project) -> Vec<u64> {
    let json = serde_json::to_value(project).expect("serialize project");
    let log = json
        .get("log")
        .and_then(|v| v.as_array())
        .expect("log is an array");
    log.iter()
        .map(|e| e.get("lamport").and_then(|v| v.as_u64()).unwrap())
        .collect()
}

#[test]
fn lamport_advances_on_local_apply() {
    let mut reg = project::ModuleRegistry::new();
    reg.register::<MinimalTestModule>();

    let mut project = Project::new();
    let view = project.create_view(&reg).unwrap();

    let mut cb = ChangeBuilder::from(&project);
    let _ = view.create_document(&mut cb, "/a".try_into().unwrap());
    project.apply_changes(cb, &reg).unwrap();

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    let _ = view.create_document(&mut cb, "/b".try_into().unwrap());
    project.apply_changes(cb, &reg).unwrap();

    let lamports = lamports_of(&project);
    // Expected entries in order:
    //   NewSession (lamport 0)
    //   Changes("/a" creation, lamport 1)
    //   Changes("/b" creation, lamport 2)
    assert_eq!(lamports, vec![0, 1, 2]);
}
