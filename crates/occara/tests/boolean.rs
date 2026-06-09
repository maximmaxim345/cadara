use occara::{
    geom::{Direction, Point},
    shape::Shape,
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
