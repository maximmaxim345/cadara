// This file includes only auto-generated bindings
// We can safely ignore all clippy warnings here
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_lifetimes)]

autocxx::include_cpp! {
    #include "shape.hpp"
    #include "geom.hpp"
    #include "internal/MakeBottle.hpp"
    safety!(unsafe)
    generate_ns!("occara")
    generate!("MakeBottle")
}

// Re-export the generated bindings
pub(crate) use ffi::*;
