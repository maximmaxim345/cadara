mod common;
use common::{minimal_test_module::MinimalTestModule, setup_project};
use serde::de::DeserializeSeed;

use project::*;

#[test]
fn test_serialize_deserialize_json() {
    let p = setup_project();

    let json = serde_json::to_string_pretty(&p.project).unwrap();

    let seed = ProjectDeserializer { registry: &p.reg };

    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    let loaded_project = seed.deserialize(deserializer).unwrap();

    let view = p.view();
    let loaded_view = loaded_project.create_view(&p.reg).unwrap();

    let docview = view.open_document(p.doc1).unwrap();
    let dview = docview
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();

    let loaded_docview = loaded_view.open_document(p.doc1).unwrap();
    let loaded_dview = loaded_docview
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();

    assert_eq!(
        dview.persistent, loaded_dview.persistent,
        "Persistent data should be serialized"
    );
    assert_eq!(
        dview.persistent_user, loaded_dview.persistent_user,
        "Persistent User data should be serialized"
    );
    assert_eq!(
        loaded_dview.session_data.num, 0,
        "Session data should not be serialized"
    );
    assert_eq!(
        loaded_dview.shared_data.num, 0,
        "Session data should not be serialized"
    );
}

#[test]
fn test_deserialize_unknown_module_json() {
    let p = setup_project();

    let json = serde_json::to_string_pretty(&p.project).unwrap();

    let empty_reg = ModuleRegistry::new();
    let seed = ProjectDeserializer {
        registry: &empty_reg,
    };

    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    assert!(seed.deserialize(deserializer).is_err());
}

#[test]
fn project_with_undo_redo_mergebranch_round_trips() {
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

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    view.open_data_by_id::<MinimalTestModule>(data_id)
        .unwrap()
        .apply_persistent(7, &mut cb);
    cb.undo();
    cb.redo();
    cb.merge_branch(BranchId::new(), project.current_branch());
    project.apply_changes(cb, &reg).unwrap();

    let json = serde_json::to_string(&project).unwrap();
    let seed = ProjectDeserializer { registry: &reg };
    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    let round_tripped = seed.deserialize(deserializer).unwrap();

    let a = project.create_view(&reg).unwrap();
    let b = round_tripped.create_view(&reg).unwrap();
    assert_eq!(
        a.open_data_by_id::<MinimalTestModule>(data_id)
            .unwrap()
            .persistent
            .num,
        b.open_data_by_id::<MinimalTestModule>(data_id)
            .unwrap()
            .persistent
            .num,
    );
}

#[test]
fn project_uuid_round_trips_through_serde() {
    // The real observable property: a deserialized replica can be merged back
    // into the original. merge_remote returns DifferentProject if the uuid
    // didn't survive the round-trip.
    let mut reg = ModuleRegistry::new();
    reg.register::<MinimalTestModule>();
    let mut project = Project::new();

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();
    let _ = view.create_document(&mut cb, "/d".try_into().unwrap());
    project.apply_changes(cb, &reg).unwrap();

    let json = serde_json::to_string(&project).unwrap();
    let seed = ProjectDeserializer { registry: &reg };
    let deserializer = &mut serde_json::Deserializer::from_str(&json);
    let round_tripped = seed.deserialize(deserializer).unwrap();

    let mut a = project;
    let b = round_tripped;
    a.merge_remote(&b, &reg)
        .expect("uuids should match after serde round-trip");
}
