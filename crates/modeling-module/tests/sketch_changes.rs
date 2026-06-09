use modeling_module::operation::sketch::{
    CircleChange, LineChange, Plane, Point2D, PrimitiveChange, Sketch, SketchChange,
    SketchPrimitive,
};
use modeling_module::operation::Operation;
use uuid::Uuid;

fn line(x1: f64, y1: f64, x2: f64, y2: f64) -> SketchPrimitive {
    SketchPrimitive::Line(modeling_module::operation::sketch::Line {
        from: Point2D::new(x1, y1),
        to: Point2D::new(x2, y2),
    })
}

#[test]
fn set_plane_updates_plane() {
    let mut s = Sketch::default();
    assert_eq!(s.plane, Plane::XY);
    s.apply_change(SketchChange::SetPlane(Plane::YZ));
    assert_eq!(s.plane, Plane::YZ);
}

#[test]
fn add_primitive_with_no_anchor_appends() {
    let mut s = Sketch::default();
    let id = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    assert_eq!(s.primitives.len(), 1);
    assert_eq!(s.primitives[0].0, id);
}

#[test]
fn add_primitive_inserts_before_anchor() {
    let mut s = Sketch::default();
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id: id_a,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::AddPrimitive {
        id: id_b,
        before: Some(id_a),
        primitive: line(1.0, 0.0, 1.0, 1.0),
    });
    assert_eq!(s.primitives[0].0, id_b);
    assert_eq!(s.primitives[1].0, id_a);
}

#[test]
fn add_primitive_unknown_anchor_appends() {
    let mut s = Sketch::default();
    let id_a = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id: id_a,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    let id_b = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id: id_b,
        before: Some(Uuid::new_v4()),
        primitive: line(1.0, 0.0, 1.0, 1.0),
    });
    assert_eq!(s.primitives.len(), 2);
    assert_eq!(s.primitives[1].0, id_b);
}

#[test]
fn add_primitive_duplicate_id_is_noop() {
    let mut s = Sketch::default();
    let id = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(2.0, 2.0, 3.0, 3.0),
    });
    assert_eq!(s.primitives.len(), 1);
}

#[test]
fn delete_primitive_removes_by_id() {
    let mut s = Sketch::default();
    let id = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::DeletePrimitive { id });
    assert!(s.primitives.is_empty());
}

#[test]
fn delete_primitive_unknown_id_is_noop() {
    let mut s = Sketch::default();
    s.apply_change(SketchChange::DeletePrimitive { id: Uuid::new_v4() });
    assert!(s.primitives.is_empty());
}

#[test]
fn reorder_primitive_moves_with_known_anchor() {
    let mut s = Sketch::default();
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let id_c = Uuid::new_v4();
    for (id, p) in [
        (id_a, line(0.0, 0.0, 1.0, 0.0)),
        (id_b, line(1.0, 0.0, 1.0, 1.0)),
        (id_c, line(1.0, 1.0, 0.0, 1.0)),
    ] {
        s.apply_change(SketchChange::AddPrimitive {
            id,
            before: None,
            primitive: p,
        });
    }
    s.apply_change(SketchChange::ReorderPrimitive {
        id: id_c,
        before: Some(id_a),
    });
    assert_eq!(
        s.primitives.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![id_c, id_a, id_b]
    );
}

#[test]
fn reorder_primitive_unknown_anchor_moves_to_end() {
    let mut s = Sketch::default();
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id: id_a,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::AddPrimitive {
        id: id_b,
        before: None,
        primitive: line(1.0, 0.0, 1.0, 1.0),
    });
    s.apply_change(SketchChange::ReorderPrimitive {
        id: id_a,
        before: Some(Uuid::new_v4()),
    });
    assert_eq!(
        s.primitives.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![id_b, id_a]
    );
}

#[test]
fn edit_primitive_line_set_from_changes_only_from() {
    let mut s = Sketch::default();
    let id = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::EditPrimitive {
        id,
        change: PrimitiveChange::Line(LineChange::SetFrom(Point2D::new(5.0, 6.0))),
    });
    let SketchPrimitive::Line(ref l) = s.primitives[0].1 else {
        panic!("expected Line")
    };
    assert_eq!((l.from.x, l.from.y), (5.0, 6.0));
    assert_eq!((l.to.x, l.to.y), (1.0, 0.0));
}

#[test]
fn edit_primitive_mismatched_variant_is_noop() {
    let mut s = Sketch::default();
    let id = Uuid::new_v4();
    s.apply_change(SketchChange::AddPrimitive {
        id,
        before: None,
        primitive: line(0.0, 0.0, 1.0, 0.0),
    });
    s.apply_change(SketchChange::EditPrimitive {
        id,
        change: PrimitiveChange::Circle(CircleChange::SetRadius(99.0)),
    });
    let SketchPrimitive::Line(ref l) = s.primitives[0].1 else {
        panic!("expected Line")
    };
    assert_eq!((l.from.x, l.from.y), (0.0, 0.0));
    assert_eq!((l.to.x, l.to.y), (1.0, 0.0));
}

#[test]
fn edit_primitive_unknown_id_is_noop() {
    let mut s = Sketch::default();
    s.apply_change(SketchChange::EditPrimitive {
        id: Uuid::new_v4(),
        change: PrimitiveChange::Line(LineChange::SetFrom(Point2D::new(1.0, 1.0))),
    });
    assert!(s.primitives.is_empty());
}
