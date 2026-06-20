use occara::{
    geom::{Point, Vector},
    shape::{Edge, Wire},
};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn test_huge_fillet_returns_err_not_crash() {
    wasm_libc::init();
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
    let cube = wire.face().extrude(&Vector::new(0.0, 0.0, 1.0));
    let edges: Vec<_> = cube.edges().collect();
    let mut builder = cube.fillet();
    for e in &edges {
        builder.add(100.0, e);
    }
    assert!(builder.build().is_err(), "expected Err, not a crash");
}
