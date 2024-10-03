use std::{sync::Arc, thread};

use occara::{
    geom::{Direction, Point},
    shape::Shape,
};

#[test]
fn test_thread_safety() {
    // This is a basic test for verifying that this library is probably thread safe.
    // Since all wrapper types function roughly in the same way, we will only test `Mesh` here.
    // This test passes while using valgrind.
    // It may still be possible that (form a type system) independent objects cause a race condition.
    let plane = Point::new(0.0, 0.0, 0.0).plane_axis_with(&Direction::z());
    let mesh = Shape::cylinder(&plane, 1.0, 1.0).mesh();

    let mesh = Arc::new(mesh);
    let handles: Vec<_> = (0..200)
        .map(|_| {
            let m = mesh.clone();
            thread::spawn(move || {
                let _ = m.vertices();
                let _ = m.indices();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    thread::spawn(|| {
        // This tests the Send trait
        std::mem::drop(mesh);
    })
    .join()
    .unwrap();
}
