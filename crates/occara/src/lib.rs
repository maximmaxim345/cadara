#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

mod ffi;

pub mod geom;
pub mod shape;

#[doc(hidden)]
pub mod internal {
    use autocxx::prelude::*;

    // This function is specifically for integration testing.
    // It builds a bottle directly using OpenCASCADE's C++ API, to compare with the Rust implementation.
    #[doc(hidden)]
    #[must_use]
    pub fn make_bottle_cpp(width: f64, height: f64, thickness: f64) -> crate::shape::Shape {
        crate::shape::Shape(crate::ffi::MakeBottle(width, height, thickness).within_box())
    }
}

#[cfg(test)]
mod tests {
    use crate::shape;

    #[test]
    fn test_vertex_coordinates() {
        let mut vertex = shape::Vertex::new();
        let (initial_x, initial_y, initial_z) = vertex.get_coordinates();

        // Check that the initial coordinates are as expected (e.g., (0.0, 0.0, 0.0) if that's the default)
        assert_eq!(initial_x, 0.0);
        assert_eq!(initial_y, 0.0);
        assert_eq!(initial_z, 0.0);

        // Set new coordinates
        let new_x = 1.0;
        let new_y = 2.0;
        let new_z = 3.0;
        vertex.set_coordinates(new_x, new_y, new_z);

        // Retrieve the coordinates after setting them
        let (updated_x, updated_y, updated_z) = vertex.get_coordinates();

        // Check that the coordinates have been updated correctly
        assert_eq!(updated_x, new_x);
        assert_eq!(updated_y, new_y);
        assert_eq!(updated_z, new_z);
    }
}
