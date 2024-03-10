#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
//! # opencascade-sys
//!
//! This crate compiles the ``OpenCASCADE`` libraries. It however does not provide any Rust bindings to
//! the ``OpenCASCADE`` libraries. For that, a high level crate 'occara' is provided.
//!
//! This crate is intended to be used as a build dependency for the 'occara' crate.
//!
//! ## Usage
//! To use this crate, add it as a build dependency in your Cargo.toml file:
//! ```toml
//! [build-dependencies]
//! opencascade-sys = { path = "../opencascade-sys" }
//! ```
//! Then, in your build.rs file, call the ``link_opencascade`` function to link the ``OpenCASCADE`` libraries
//! after linking any other libraries that depend on ``OpenCASCADE``.
//! ```no_run
//! // ... linking other libraries, by generating and linking with the appropriate binding
//! opencascade_sys::link_opencascade()
//! ```
use std::path::PathBuf;

/// Returns the path to the ``OpenCASCADE`` library directory.
///
/// This function returns the system library directory for the ``OpenCASCADE`` libraries.
/// This director contains the build static libraries for the ``OpenCASCADE`` libraries.
/// For simple linking, use the `link_opencascade` function, which will automatically link the
/// ``OpenCASCADE`` libraries in the correct order.
#[must_use]
pub fn lib_dir() -> PathBuf {
    PathBuf::from(env!("OPENCASCADE_LIB_DIR"))
}

/// Returns the path to the ``OpenCASCADE`` include directory.
///
/// This function returns the system include directory for the ``OpenCASCADE`` libraries. It should be
/// used to include the ``OpenCASCADE`` headers in the build process, for example when generating
/// bindings to the ``OpenCASCADE`` libraries.
#[must_use]
pub fn include_dir() -> PathBuf {
    PathBuf::from(env!("OPENCASCADE_INCLUDE_DIR"))
}

/// Link the ``OpenCASCADE`` libraries against the current build.
///
/// This function should be called from the build.rs file of the crate that depends on this crate.
/// It will link the ``OpenCASCADE`` libraries statically against the current build, using the
/// cargo:rustc-link-search=native and cargo:rustc-link-lib=static directives.
///
/// Due to the way the linker works, this function must be called after linking any other
/// libraries that depend on ``OpenCASCADE``.
#[allow(clippy::missing_panics_doc)] // panic should not have any effects, since this is a build script
pub fn link_opencascade() {
    let lib_dir = lib_dir();
    // now link statically with all the OCCT libraries (in correct order)
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
    // I dont't know why this order works, but it does, so I'm not going to mess with it for now
    let lib_linking_order = vec![
        "TKBO",
        "TKBool",
        "TKBRep",
        "TKFeat",
        "TKFillet",
        "TKG2d",
        "TKG3d",
        "TKGeomAlgo",
        "TKGeomBase",
        "TKHLR",
        "TKMath",
        "TKMesh",
        "TKOffset",
        "TKPrim",
        "TKShHealing",
        "TKTopAlgo",
        "TKernel",
    ];

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    for lib in lib_linking_order {
        if target_os == "windows" {
            // I don't know why this is needed, but it is
            println!("cargo:rustc-link-lib=static:-whole-archive={lib}");
        } else {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }

    if target_os == "windows" {
        // Also link with the user32 library, which is for some reason needed
        println!("cargo:rustc-link-lib=user32");
    }
}
