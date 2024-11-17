use occara::{
    geom::{Direction, Point},
    shape::Shape,
};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn test_mesh_cylinder() {
    wasm_libc::init();
    let plane = Point::new(0.0, 0.0, 0.0).plane_axis_with(&Direction::z());
    let mesh = Shape::cylinder(&plane, 1.0, 1.0).mesh();
    let vertices = mesh.vertices();
    let indices = mesh.indices();

    // Check mesh is not empty
    assert!(vertices.len() > 50, "Too few vertices");
    assert!(indices.len() > 50, "Too few indices");

    // Check indices are valid
    assert!(indices.iter().all(|&i| i < vertices.len()), "Invalid index");

    // Check vertices form a cylinder
    let (min_z, max_z) = vertices
        .iter()
        .map(|v| v.z())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), z| {
            (min.min(z), max.max(z))
        });

    assert!((max_z - min_z - 1.0).abs() < 1e-6, "Incorrect height");

    let max_radius = vertices
        .iter()
        .map(|v| (v.x().powi(2) + v.y().powi(2)).sqrt())
        .fold(0.0, f64::max);

    assert!((max_radius - 1.0).abs() < 1e-6, "Incorrect radius");
}
