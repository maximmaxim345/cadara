use occara::internal::{make_bottle_cpp, make_bottle_rust};
use std::sync::Arc;
use std::thread;
use wasm_bindgen_test::*;

#[test]
fn test_simple_edge_iteration() {
    use occara::geom::{Direction, Point};
    use occara::shape::Shape;

    println!("Creating simple cylinder...");
    let axis = Point::new(0.0, 0.0, 0.0).plane_axis_with(&Direction::z());
    let cylinder = Shape::cylinder(&axis, 10.0, 20.0);
    println!("Cylinder created");

    println!("Iterating edges...");
    let mut count = 0;
    let mut iter = cylinder.edges();
    loop {
        println!("About to call next for edge {}...", count + 1);
        match iter.next() {
            Some(edge) => {
                count += 1;
                println!("Got edge {}", count);
                drop(edge);
            }
            None => {
                println!("No more edges");
                break;
            }
        }
    }
    println!("Found {} edges on cylinder", count);
}

#[test]
fn test_make_bottle_cpp_only() {
    const WIDTH: f64 = 50.0;
    const HEIGHT: f64 = 70.0;
    const THICKNESS: f64 = 30.0;

    // Test that the C++ implementation works
    let bottle = make_bottle_cpp(WIDTH, HEIGHT, THICKNESS);
    let mass = bottle.mass();
    println!("C++ Bottle mass: {}", mass);
    assert!(mass > 6900.0 && mass < 7000.0); // Expected ~6960
}

#[test]
fn test_make_bottle_rust_minimal() {
    use occara::geom::{Direction, Point, Transformation, Vector};
    use occara::shape::{Edge, Wire};

    const WIDTH: f64 = 50.0;
    const HEIGHT: f64 = 70.0;
    const THICKNESS: f64 = 30.0;

    println!("Creating points...");
    let point1 = Point::new(-WIDTH / 2.0, 0.0, 0.0);
    let point2 = Point::new(-WIDTH / 2.0, -THICKNESS / 4.0, 0.0);
    let point3 = Point::new(0.0, -THICKNESS / 2.0, 0.0);
    let point4 = Point::new(WIDTH / 2.0, -THICKNESS / 4.0, 0.0);
    let point5 = Point::new(WIDTH / 2.0, 0.0, 0.0);
    println!("Points created");

    println!("Creating edges...");
    let arc_of_circle = Edge::arc_of_circle(&point2, &point3, &point4);
    let segment1 = Edge::line(&point1, &point2);
    let segment2 = Edge::line(&point4, &point5);
    println!("Edges created");

    println!("Creating wire...");
    let wire = Wire::new(&[&segment1, &arc_of_circle, &segment2]);
    println!("Wire created successfully!");

    println!("Creating transformation...");
    let axis = Point::origin().axis_with(&Direction::x());
    println!("Axis created");
    let transformation = Transformation::mirror(&axis);
    println!("Transformation created");

    println!("Applying transformation...");
    let mirrored_wire = transformation.apply(&wire);
    println!("Mirrored wire created");

    println!("Combining wires...");
    let bottle_profile = Wire::new(&[&wire, &mirrored_wire]);
    println!("Bottle profile created");

    println!("Creating face...");
    let face_profile = bottle_profile.face();
    println!("Face created");

    println!("Extruding...");
    let extrude_vec = Vector::new(0.0, 0.0, HEIGHT);
    let body = face_profile.extrude(&extrude_vec);
    println!("Body extruded");

    println!("Getting edges iterator...");
    let mut iter = body.edges();
    println!("Iterator created");

    println!("Counting edges...");
    let mut count = 0;
    loop {
        println!("Calling iter.next() for edge {}...", count + 1);
        match iter.next() {
            Some(_edge) => {
                count += 1;
                println!("Got edge {}", count);
            }
            None => {
                println!("No more edges");
                break;
            }
        }
    }
    println!("Found {} edges", count);

    println!("Test completed successfully!");
}

#[test]
#[ignore] // Don't run this test by default, since its quite slow
fn test_make_bottle() {
    const WIDTH: f64 = 50.0;
    const HEIGHT: f64 = 70.0;
    const THICKNESS: f64 = 30.0;
    const EPSILON: f64 = 1e-6;

    let bottle_rust = thread::spawn(move || make_bottle_rust(WIDTH, HEIGHT, THICKNESS));
    let bottle_cpp = thread::spawn(move || make_bottle_cpp(WIDTH, HEIGHT, THICKNESS));

    let bottle_rust = bottle_rust.join().unwrap();
    let bottle_cpp = bottle_cpp.join().unwrap();

    let bottle_rust = Arc::new(bottle_rust);
    let bottle_rust2 = bottle_rust.clone();
    let bottle_cpp = Arc::new(bottle_cpp);
    let bottle_cpp2 = bottle_cpp.clone();

    let error_rust_cpp = thread::spawn(move || bottle_rust.subtract(&bottle_cpp).mass());
    let error_cpp_rust = thread::spawn(move || bottle_cpp2.subtract(&bottle_rust2).mass());

    assert!(error_rust_cpp.join().unwrap() < EPSILON);
    assert!(error_cpp_rust.join().unwrap() < EPSILON);
}

#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_make_bottle_wasm() {
    // TODO(wasm32): This test fails, since exceptions are broken
    wasm_libc::init();
    const WIDTH: f64 = 50.0;
    const HEIGHT: f64 = 70.0;
    const THICKNESS: f64 = 30.0;
    const EPSILON: f64 = 1e-6;

    let bottle_rust = make_bottle_rust(WIDTH, HEIGHT, THICKNESS);
    let bottle_cpp = make_bottle_cpp(WIDTH, HEIGHT, THICKNESS);

    let error_rust_cpp = bottle_rust.subtract(&bottle_cpp).mass();
    let error_cpp_rust = bottle_cpp.subtract(&bottle_rust).mass();

    assert!(error_rust_cpp < EPSILON);
    assert!(error_cpp_rust < EPSILON);
}
