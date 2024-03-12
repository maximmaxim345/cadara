#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

mod ffi;

pub mod shape {
    use super::ffi;
    use autocxx::prelude::*;
    use std::pin::Pin;

    pub struct Vertex(UniquePtr<ffi::shape::Vertex>);

    impl Vertex {
        #[must_use]
        pub fn new() -> Self {
            Self(ffi::shape::Vertex::new(0.0, 0.0, 0.0).within_unique_ptr())
        }

        pub fn set_coordinates(&mut self, x: f64, y: f64, z: f64) {
            self.0.as_mut().unwrap().set_coordinates(x, y, z);
        }

        #[must_use]
        pub fn get_coordinates(&self) -> (f64, f64, f64) {
            let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
            self.0
                .get_coordinates(Pin::new(&mut x), Pin::new(&mut y), Pin::new(&mut z));
            (x, y, z)
        }
    }

    impl Default for Vertex {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shape;
    #[test]
    fn test_simple() {
        let _shape = crate::ffi::MakeBottle(50.0, 70.0, 30.0);
    }

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
