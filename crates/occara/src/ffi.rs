#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
// Run 'touch cpp' and 'bear -- cargo build' in this crates directory for autocompleation of c++ source files

autocxx::include_cpp! {
    #include "MakeBottle.hpp"
    #include "shape/vertex.hpp"
    safety!(unsafe)
    generate_ns!("shape")
}

#[cxx::bridge]
mod ffi2 {
    unsafe extern "C++" {
        include!("TopoDS_Shape.hxx");
        include!("MakeBottle.hpp");
        type TopoDS_Shape;

        fn MakeBottle(width: f64, height: f64, thickness: f64) -> UniquePtr<TopoDS_Shape>;
    }
}

// Re-export the generated bindings
pub(crate) use ffi::*;
pub(crate) use ffi2::*;
