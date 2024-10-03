#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
// We can not implement Send on the underlying autocxx managed raw pointer, therefore this
// lint is not applicable here.
#![allow(clippy::non_send_fields_in_send_ty)]

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
