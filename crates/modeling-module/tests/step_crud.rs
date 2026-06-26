use modeling_module::operation::extrude::{Extrude, ExtrudeChange, ExtrudeMode};
use modeling_module::operation::fillet::{Fillet, FilletChange};
use modeling_module::operation::sketch::{Plane, Sketch, SketchChange};
use modeling_module::operation::ModelingOperation;
use modeling_module::persistent_data::{
    Create, Delete, ModelingTransaction, PersistentData, Rename, Reorder,
};
use module::DataSection;
use uuid::Uuid;

fn op_sketch() -> ModelingOperation {
    ModelingOperation::Sketch(Sketch::default())
}

fn create(id: Uuid, name: &str, before: Option<Uuid>) -> ModelingTransaction {
    ModelingTransaction::Create(Create {
        id,
        before,
        name: name.to_string(),
        operation: op_sketch(),
    })
}

#[test]
fn create_with_no_anchor_appends() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(create(id, "s1", None));
    assert_eq!(pd.steps().len(), 1);
    assert_eq!(pd.steps()[0].id, id);
}

#[test]
fn create_with_known_anchor_inserts_before() {
    let mut pd = PersistentData::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    pd.apply(create(a, "a", None));
    pd.apply(create(b, "b", Some(a)));
    assert_eq!(pd.steps()[0].id, b);
    assert_eq!(pd.steps()[1].id, a);
}

#[test]
fn create_with_unknown_anchor_appends() {
    let mut pd = PersistentData::default();
    let a = Uuid::new_v4();
    pd.apply(create(a, "a", None));
    let b = Uuid::new_v4();
    pd.apply(create(b, "b", Some(Uuid::new_v4())));
    assert_eq!(pd.steps().last().unwrap().id, b);
}

#[test]
fn create_with_duplicate_id_is_noop() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(create(id, "a", None));
    pd.apply(create(id, "b", None));
    assert_eq!(pd.steps().len(), 1);
}

#[test]
fn delete_by_known_id_removes() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(create(id, "a", None));
    pd.apply(ModelingTransaction::Delete(Delete { id }));
    assert!(pd.steps().is_empty());
}

#[test]
fn delete_by_unknown_id_is_noop() {
    let mut pd = PersistentData::default();
    pd.apply(ModelingTransaction::Delete(Delete { id: Uuid::new_v4() }));
    assert!(pd.steps().is_empty());
}

#[test]
fn rename_changes_only_name() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(create(id, "old", None));
    pd.apply(ModelingTransaction::Rename(Rename {
        id,
        new_name: "new".into(),
    }));
    assert_eq!(pd.steps()[0].name, "new");
}

#[test]
fn rename_unknown_id_is_noop() {
    let mut pd = PersistentData::default();
    pd.apply(ModelingTransaction::Rename(Rename {
        id: Uuid::new_v4(),
        new_name: "x".into(),
    }));
    assert!(pd.steps().is_empty());
}

#[test]
fn reorder_with_known_anchor_moves() {
    let mut pd = PersistentData::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    for (id, n) in [(a, "a"), (b, "b"), (c, "c")] {
        pd.apply(create(id, n, None));
    }
    pd.apply(ModelingTransaction::Reorder(Reorder {
        id: c,
        before: Some(a),
    }));
    assert_eq!(
        pd.steps().iter().map(|s| s.id).collect::<Vec<_>>(),
        vec![c, a, b]
    );
}

#[test]
fn reorder_with_unknown_anchor_moves_to_end() {
    let mut pd = PersistentData::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    pd.apply(create(a, "a", None));
    pd.apply(create(b, "b", None));
    pd.apply(ModelingTransaction::Reorder(Reorder {
        id: a,
        before: Some(Uuid::new_v4()),
    }));
    assert_eq!(
        pd.steps().iter().map(|s| s.id).collect::<Vec<_>>(),
        vec![b, a]
    );
}

#[test]
fn reorder_unknown_id_is_noop() {
    let mut pd = PersistentData::default();
    let a = Uuid::new_v4();
    pd.apply(create(a, "a", None));
    pd.apply(ModelingTransaction::Reorder(Reorder {
        id: Uuid::new_v4(),
        before: None,
    }));
    assert_eq!(pd.steps().len(), 1);
}

#[test]
fn edit_sketch_set_plane_dispatches_to_sketch() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(create(id, "s", None));
    pd.apply(ModelingTransaction::EditSketch {
        step_id: id,
        change: SketchChange::SetPlane(Plane::YZ),
    });
    let ModelingOperation::Sketch(ref s) = pd.steps()[0].operation else {
        panic!()
    };
    assert_eq!(s.plane, Plane::YZ);
}

#[test]
fn edit_sketch_on_extrude_step_is_noop() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(ModelingTransaction::Create(Create {
        id,
        before: None,
        name: "e".into(),
        operation: ModelingOperation::Extrude(Extrude::default()),
    }));
    pd.apply(ModelingTransaction::EditSketch {
        step_id: id,
        change: SketchChange::SetPlane(Plane::YZ),
    });
    let ModelingOperation::Extrude(_) = pd.steps()[0].operation else {
        panic!()
    };
}

#[test]
fn edit_extrude_dispatches_to_extrude() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(ModelingTransaction::Create(Create {
        id,
        before: None,
        name: "e".into(),
        operation: ModelingOperation::Extrude(Extrude::default()),
    }));
    pd.apply(ModelingTransaction::EditExtrude {
        step_id: id,
        change: ExtrudeChange::SetMode(ExtrudeMode::Subtract),
    });
    let ModelingOperation::Extrude(ref e) = pd.steps()[0].operation else {
        panic!()
    };
    assert_eq!(e.mode, ExtrudeMode::Subtract);
}

#[test]
fn edit_fillet_dispatches_to_fillet() {
    let mut pd = PersistentData::default();
    let id = Uuid::new_v4();
    pd.apply(ModelingTransaction::Create(Create {
        id,
        before: None,
        name: "f".into(),
        operation: ModelingOperation::Fillet(Fillet::default()),
    }));
    pd.apply(ModelingTransaction::EditFillet {
        step_id: id,
        change: FilletChange::SetRadius(2.5),
    });
    let ModelingOperation::Fillet(ref f) = pd.steps()[0].operation else {
        panic!()
    };
    assert_eq!(f.radius, 2.5);
}

#[test]
fn edit_with_unknown_step_id_is_noop() {
    let mut pd = PersistentData::default();
    pd.apply(ModelingTransaction::EditSketch {
        step_id: Uuid::new_v4(),
        change: SketchChange::SetPlane(Plane::YZ),
    });
    assert!(pd.steps().is_empty());
}
