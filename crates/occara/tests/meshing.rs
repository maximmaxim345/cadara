use occara::{
    geom::{Direction, Point},
    shape::Shape,
};

#[test]
fn test_mesh_cylinder() {
    let plane = Point::new(0.0, 0.0, 0.0).plane_axis_with(&Direction::z());
    let mesh = Shape::cylinder(&plane, 1.0, 1.0).mesh();
    let vertices = mesh.vertices();
    let indices = mesh.indices();

    // Just check that the mesh is not empty
    // It will be obvious if the mesh is wrong when rendering
    // TODO: this should be a benchmark instead
    assert!(vertices.len() > 50);
    assert!(indices.len() > 50);
}
