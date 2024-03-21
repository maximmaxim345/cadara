// This file includes only auto-generated bindings
// We can safely ignore all clippy warnings here
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_lifetimes)]
// Run 'touch cpp' and 'bear -- cargo build' in this crates directory for autocompleation of c++ source files

autocxx::include_cpp! {
    #include "gp_Pnt.hxx"
    #include "gp_Dir.hxx"
    #include "gp_Ax1.hxx"
    #include "MakeBottle.hpp"
    #include "shape.hpp"
    #include "geom.hpp"
    safety!(unsafe)
    generate_ns!("occara")
    generate!("gp_Pnt")
    generate!("gp_Dir")
    generate!("gp_Ax1")
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
