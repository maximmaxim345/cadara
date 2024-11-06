use occara::internal::{make_bottle_cpp, make_bottle_rust};
use std::sync::Arc;
use std::thread;
use wasm_bindgen_test::*;

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
