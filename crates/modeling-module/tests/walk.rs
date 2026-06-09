use modeling_module::operation::extrude::{Extrude, ExtrudeDirection, ExtrudeMode};
use modeling_module::operation::fillet::{EdgeRef, Fillet, FilletTarget};
use modeling_module::operation::sketch::{Line, Plane, Point2D, Sketch, SketchPrimitive};
use modeling_module::operation::ModelingOperation;
use modeling_module::persistent_data::{Create, ModelingTransaction, PersistentData};
use module::DataSection;
use uuid::Uuid;

fn add(pd: &mut PersistentData, id: Uuid, op: ModelingOperation) {
    pd.apply(ModelingTransaction::Create(Create {
        id,
        before: None,
        name: "x".into(),
        operation: op,
    }));
}

fn square_sketch_xy() -> Sketch {
    let prims = vec![
        (
            Uuid::new_v4(),
            SketchPrimitive::Line(Line {
                from: Point2D::new(0.0, 0.0),
                to: Point2D::new(1.0, 0.0),
            }),
        ),
        (
            Uuid::new_v4(),
            SketchPrimitive::Line(Line {
                from: Point2D::new(1.0, 0.0),
                to: Point2D::new(1.0, 1.0),
            }),
        ),
        (
            Uuid::new_v4(),
            SketchPrimitive::Line(Line {
                from: Point2D::new(1.0, 1.0),
                to: Point2D::new(0.0, 1.0),
            }),
        ),
        (
            Uuid::new_v4(),
            SketchPrimitive::Line(Line {
                from: Point2D::new(0.0, 1.0),
                to: Point2D::new(0.0, 0.0),
            }),
        ),
    ];
    Sketch {
        plane: Plane::XY,
        primitives: prims,
    }
}

fn bbox(pd: &PersistentData) -> (f64, f64, f64, f64, f64, f64) {
    let verts = pd.shape().mesh().vertices();
    if verts.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let (mut minx, mut maxx) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut miny, mut maxy) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut minz, mut maxz) = (f64::INFINITY, f64::NEG_INFINITY);
    for v in &verts {
        minx = minx.min(v.x());
        maxx = maxx.max(v.x());
        miny = miny.min(v.y());
        maxy = maxy.max(v.y());
        minz = minz.min(v.z());
        maxz = maxz.max(v.z());
    }
    (minx, maxx, miny, maxy, minz, maxz)
}

#[test]
fn empty_steps_produces_empty_mesh() {
    let pd = PersistentData::default();
    assert!(pd.shape().mesh().vertices().is_empty());
}

#[test]
fn sketch_alone_produces_empty_mesh() {
    let mut pd = PersistentData::default();
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Sketch(square_sketch_xy()),
    );
    assert!(pd.shape().mesh().vertices().is_empty());
}

#[test]
fn add_extrude_on_empty_seeds_body() {
    let mut pd = PersistentData::default();
    let sketch_id = Uuid::new_v4();
    add(
        &mut pd,
        sketch_id,
        ModelingOperation::Sketch(square_sketch_xy()),
    );
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id,
            depth: 2.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    let (minx, maxx, miny, maxy, minz, maxz) = bbox(&pd);
    assert!(
        (maxx - minx - 1.0).abs() < 1e-6,
        "x extent != 1: {}",
        maxx - minx
    );
    assert!(
        (maxy - miny - 1.0).abs() < 1e-6,
        "y extent != 1: {}",
        maxy - miny
    );
    assert!(
        (maxz - minz - 2.0).abs() < 1e-6,
        "z extent != 2: {}",
        maxz - minz
    );
}

#[test]
fn subtract_on_empty_is_noop() {
    let mut pd = PersistentData::default();
    let sketch_id = Uuid::new_v4();
    add(
        &mut pd,
        sketch_id,
        ModelingOperation::Sketch(square_sketch_xy()),
    );
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id,
            depth: 2.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Subtract,
        }),
    );
    assert!(pd.shape().mesh().vertices().is_empty());
}

#[test]
fn extrude_with_unknown_sketch_id_is_skipped() {
    let mut pd = PersistentData::default();
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id: Uuid::new_v4(),
            depth: 1.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    assert!(pd.shape().mesh().vertices().is_empty());
}

#[test]
fn extrude_referencing_non_sketch_step_is_skipped() {
    let mut pd = PersistentData::default();
    let fillet_id = Uuid::new_v4();
    add(
        &mut pd,
        fillet_id,
        ModelingOperation::Fillet(Fillet::default()),
    );
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id: fillet_id,
            depth: 1.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    assert!(pd.shape().mesh().vertices().is_empty());
}

#[test]
fn fillet_whole_body_shrinks_bounding_box_or_equal() {
    let mut pd = PersistentData::default();
    let sketch_id = Uuid::new_v4();
    add(
        &mut pd,
        sketch_id,
        ModelingOperation::Sketch(square_sketch_xy()),
    );
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id,
            depth: 1.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    let (minx0, maxx0, _, _, _, _) = bbox(&pd);
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Fillet(Fillet {
            radius: 0.1,
            target: FilletTarget::WholeBody,
        }),
    );
    let (minx1, maxx1, _, _, _, _) = bbox(&pd);
    assert!(
        maxx1 - minx1 <= maxx0 - minx0 + 1e-9,
        "filleted x-extent should be <= unfilleted"
    );
}

#[test]
fn fillet_with_all_out_of_range_edges_is_noop() {
    let mut pd = PersistentData::default();
    let sketch_id = Uuid::new_v4();
    add(
        &mut pd,
        sketch_id,
        ModelingOperation::Sketch(square_sketch_xy()),
    );
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id,
            depth: 1.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    let before = bbox(&pd);
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Fillet(Fillet {
            radius: 0.1,
            target: FilletTarget::Edges(vec![EdgeRef { index: 9999 }]),
        }),
    );
    let after = bbox(&pd);
    assert_eq!(before, after, "out-of-range fillet must not change body");
}

#[test]
fn yz_sketch_extrudes_along_x() {
    let mut pd = PersistentData::default();
    let sketch_id = Uuid::new_v4();
    let mut s = square_sketch_xy();
    s.plane = Plane::YZ;
    add(&mut pd, sketch_id, ModelingOperation::Sketch(s));
    add(
        &mut pd,
        Uuid::new_v4(),
        ModelingOperation::Extrude(Extrude {
            sketch_id,
            depth: 2.0,
            direction: ExtrudeDirection::Normal,
            mode: ExtrudeMode::Add,
        }),
    );
    let (minx, maxx, miny, maxy, minz, maxz) = bbox(&pd);
    assert!(
        (maxx - minx - 2.0).abs() < 1e-6,
        "x should extend along extrude direction (depth=2): got {}",
        maxx - minx
    );
    assert!(
        (maxy - miny - 1.0).abs() < 1e-6,
        "y extent should be unit: got {}",
        maxy - miny
    );
    assert!(
        (maxz - minz - 1.0).abs() < 1e-6,
        "z extent should be unit: got {}",
        maxz - minz
    );
}
