# occara

occara provides high-level Rust bindings for the OpenCASCADE CAD library. It is an internally used library for CADara.

## Structure

occara consists of:
- A higher-level C++ wrapper around OpenCASCADE, written in an easily parse-able subset of C++. This allows performance critical CADara-specific code to directly use optimized OpenCASCADE data structures, avoiding overhead from the C++ to Rust bridge.
- Rust bindings for the C++ wrapper, automatically generated using autocxx
- A thin Rust library layer to provide a safe, idiomatic Rust API

## Status & Todos
- Error handling is not complete yet. The C++ wrapper should catch all exceptions and pass them to Rust, since autocxx does not support C++ exceptions directly.
- This library does not feature all OpenCASCADE functionality yet. It is being developed incrementally as needed by CADara.
- in the future, this API could be made accessible to CADara plugins. This would enable plugins to implement custom CAD operations and functionality, while being sandboxed from the core CADara application for security and stability.
