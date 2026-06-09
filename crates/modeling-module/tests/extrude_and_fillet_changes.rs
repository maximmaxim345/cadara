use modeling_module::operation::extrude::{Extrude, ExtrudeChange, ExtrudeDirection, ExtrudeMode};
use modeling_module::operation::Operation;
use uuid::Uuid;

fn ex() -> Extrude {
    Extrude {
        sketch_id: Uuid::nil(),
        depth: 1.0,
        direction: ExtrudeDirection::Normal,
        mode: ExtrudeMode::Add,
    }
}

#[test]
fn extrude_set_depth_changes_only_depth() {
    let mut e = ex();
    e.apply_change(ExtrudeChange::SetDepth(7.5));
    assert_eq!(e.depth, 7.5);
    assert_eq!(e.direction, ExtrudeDirection::Normal);
    assert_eq!(e.mode, ExtrudeMode::Add);
}

#[test]
fn extrude_set_sketch_id_changes_only_sketch_id() {
    let mut e = ex();
    let new_id = Uuid::new_v4();
    e.apply_change(ExtrudeChange::SetSketchId(new_id));
    assert_eq!(e.sketch_id, new_id);
    assert_eq!(e.depth, 1.0);
}

#[test]
fn extrude_set_direction_changes_only_direction() {
    let mut e = ex();
    e.apply_change(ExtrudeChange::SetDirection(ExtrudeDirection::Symmetric));
    assert_eq!(e.direction, ExtrudeDirection::Symmetric);
    assert_eq!(e.depth, 1.0);
}

#[test]
fn extrude_set_mode_changes_only_mode() {
    let mut e = ex();
    e.apply_change(ExtrudeChange::SetMode(ExtrudeMode::Subtract));
    assert_eq!(e.mode, ExtrudeMode::Subtract);
    assert_eq!(e.depth, 1.0);
}
