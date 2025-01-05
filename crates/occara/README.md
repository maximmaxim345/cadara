# occara

occara provides high-level Rust bindings for the OpenCASCADE B-Rep library. It is internally used by CADara to manipulate 3D models.

## Structure

occara consists of:
- A higher-level C++ wrapper around OpenCASCADE, written in an easily parse-able subset of C++. This allows performance critical CADara-specific code to directly use optimized OpenCASCADE data structures, avoiding overhead from the C++ to Rust bridge.
- Rust bindings for the C++ wrapper, automatically generated using autocxx
- A thin Rust library layer to provide a safe, idiomatic Rust API

## Status & Todos
- Error handling is not implemented yet. The C++ wrapper should catch all exceptions and pass them to Rust, since autocxx does not support C++ exceptions.
- This library does not feature all OpenCASCADE functionality yet. It is being developed incrementally as needed by CADara.
- in the future, this API could be made accessible to CADara plugins. This would enable plugins to implement custom CAD operations and functionality, while being sandboxed from the core CADara application for security and stability.

## Development

To get autocomplete to work on the C++ code, run `touch cpp` and `bear -- cargo build` in this crates' directory. This will generate a compile_commands.json file. ('touch cpp' will ensure that the C++ code is recompiled)
