use occara::{
    geom::{Direction, Point},
    shape::{Edge, Shape, Wire},
};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn test_subtract_smaller_cylinder_from_bigger() {
    wasm_libc::init();
    let plane = Point::new(0.0, 0.0, 0.0).plane_axis_with(&Direction::z());
    let big = Shape::cylinder(&plane, 2.0, 1.0);
    let small = Shape::cylinder(&plane, 1.0, 1.0);

    let result = big.subtract(&small);
    let mesh = result.mesh();
    let verts = mesh.vertices();

    let max_r = verts
        .iter()
        .map(|v| (v.x().powi(2) + v.y().powi(2)).sqrt())
        .fold(0.0, f64::max);
    let min_r = verts
        .iter()
        .filter(|v| (v.x().powi(2) + v.y().powi(2)).sqrt() > 0.01)
        .map(|v| (v.x().powi(2) + v.y().powi(2)).sqrt())
        .fold(f64::INFINITY, f64::min);

    assert!((max_r - 2.0).abs() < 1e-6, "outer radius wrong: {max_r}");
    assert!((min_r - 1.0).abs() < 1e-6, "inner radius wrong: {min_r}");
}

#[wasm_bindgen_test(unsupported = test)]
fn test_circle_edge_forms_closed_wire() {
    wasm_libc::init();
    let center = Point::new(0.0, 0.0, 0.0);
    let normal = Direction::z();
    let edge = Edge::circle(&center, &normal, 1.5);

    let wire = Wire::new(&[&edge]);
    let face = wire.face();

    let mesh = face
        .extrude(&occara::geom::Vector::new(0.0, 0.0, 0.5))
        .mesh();
    let verts = mesh.vertices();

    let max_r = verts
        .iter()
        .map(|v| (v.x().powi(2) + v.y().powi(2)).sqrt())
        .fold(0.0, f64::max);
    let min_z = verts.iter().map(|v| v.z()).fold(f64::INFINITY, f64::min);
    let max_z = verts
        .iter()
        .map(|v| v.z())
        .fold(f64::NEG_INFINITY, f64::max);

    assert!((max_r - 1.5).abs() < 1e-6, "circle radius wrong: {max_r}");
    assert!(
        (max_z - min_z - 0.5).abs() < 1e-6,
        "height wrong: {}",
        max_z - min_z
    );
}

#[wasm_bindgen_test(unsupported = test)]
fn test_face_edges_returns_face_boundary() {
    wasm_libc::init();
    // A square face has 4 boundary edges. (BREP may include seam edges; we
    // just assert at least 4 and that all edges are valid by counting them.)
    let p1 = Point::new(0.0, 0.0, 0.0);
    let p2 = Point::new(1.0, 0.0, 0.0);
    let p3 = Point::new(1.0, 1.0, 0.0);
    let p4 = Point::new(0.0, 1.0, 0.0);
    let wire = Wire::new(&[
        &Edge::line(&p1, &p2),
        &Edge::line(&p2, &p3),
        &Edge::line(&p3, &p4),
        &Edge::line(&p4, &p1),
    ]);
    let face = wire.face();
    let edge_count = face.edges().count();
    assert!(
        edge_count >= 4,
        "expected at least 4 face edges, got {edge_count}"
    );
}
