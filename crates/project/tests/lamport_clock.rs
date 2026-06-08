mod common;

use common::*;
use project::{ChangeBuilder, ModuleRegistry, Project, UserId};

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

#[test]
fn merge_remote_advances_clock_past_max_remote_lamport() {
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();
    let mut a = Project::new();

    let mut cb = ChangeBuilder::from(&a);
    let view = a.create_view(&reg).unwrap();
    let _doc = *view.create_document(&mut cb, "/d".try_into().unwrap());
    a.apply_changes(cb, &reg).unwrap();

    // Fork.
    let mut b = a.fork_replica(UserId::new());

    // B advances its clock by applying many local ops.
    for i in 0..10 {
        let mut cb = ChangeBuilder::from(&b);
        let view = b.create_view(&reg).unwrap();
        let _ = view.create_document(&mut cb, format!("/x{i}").as_str().try_into().unwrap());
        b.apply_changes(cb, &reg).unwrap();
    }

    let b_max = lamports_of(&b).into_iter().max().unwrap();

    // A merges B. A's clock should jump past B's max.
    a.merge_remote(&b, &reg).unwrap();

    // A's next local op must have lamport > b_max.
    let mut cb = ChangeBuilder::from(&a);
    let view = a.create_view(&reg).unwrap();
    let _ = view.create_document(&mut cb, "/post-merge".try_into().unwrap());
    a.apply_changes(cb, &reg).unwrap();

    let a_max = lamports_of(&a).into_iter().max().unwrap();
    assert!(a_max > b_max, "a_max={a_max} should exceed b_max={b_max}");
}
