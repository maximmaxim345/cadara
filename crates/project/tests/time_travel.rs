mod common;

use common::*;
use project::{ChangeBuilder, ModuleRegistry, Project};

#[test]
fn create_view_at_returns_historical_state() {
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

    // Edit a series of values; capture the lamport of each edit by inspecting
    // the log post-apply.
    let mut points: Vec<(i32, u64)> = Vec::new();
    for v in [3, 7, 21] {
        let mut cb = ChangeBuilder::from(&project);
        let view = project.create_view(&reg).unwrap();
        view.open_data_by_id::<MinimalTestModule>(data_id)
            .unwrap()
            .apply_persistent(v, &mut cb);
        project.apply_changes(cb, &reg).unwrap();
        let t = project_max_lamport(&project);
        points.push((v, t));
    }

    // View at the lamport of each edit; expect that value.
    let branch = project.current_branch();
    for (v, t) in &points {
        let view = project.create_view_at(&reg, branch, *t).unwrap();
        let n = view
            .open_data_by_id::<MinimalTestModule>(data_id)
            .unwrap()
            .persistent
            .num;
        assert_eq!(n, *v, "view at lamport {t} should see value {v}");
    }

    // Viewing at a lamport before the first edit but after setup gives default (0).
    let pre_edit_lamport = points[0].1 - 1;
    let view = project
        .create_view_at(&reg, branch, pre_edit_lamport)
        .unwrap();
    assert_eq!(
        view.open_data_by_id::<MinimalTestModule>(data_id)
            .unwrap()
            .persistent
            .num,
        0,
        "view before any edits should see default"
    );
}

fn project_max_lamport(p: &Project) -> u64 {
    let json = serde_json::to_value(p).unwrap();
    json.get("log")
        .and_then(|v| v.as_array())
        .unwrap()
        .iter()
        .map(|e| e.get("lamport").and_then(|x| x.as_u64()).unwrap())
        .max()
        .unwrap()
}
