#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

mod ffi;

pub mod geom;
pub mod shape;

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
