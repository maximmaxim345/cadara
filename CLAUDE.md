# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CADara is an early-stage parametric CAD application built with Rust, OpenCASCADE, iced, and wgpu. The project uses a modular architecture with:

- **Projects**: Containers for documents with version control
- **Documents**: Collections of data
- **Data**: Described by modules, defining their structure and operations
- **Modules**: Define data structures and operations (see `module` trait)
- **Workspaces**: Used to "open" and work with data, providing associated tools (for modifying data) and viewport plugins (for UI/visualization)

## Build System & Commands

CADara uses `cargo-make` as its primary build tool. All commands should be run from the repository root.

**Run frequently:** `cargo make verify`

Run this after making major changes and always before committing. It runs tests, linting, docs, formatting, spell-check, and dependency checks. Catching issues early saves time.

**For comprehensive verification:** `cargo make verify-all`

Includes everything in `verify` plus slower tests (e.g., comparing Rust vs C++ make_bottle implementations). Use before major commits or when changing core functionality.

### Essential Commands

**Individual verification tasks:**
```bash
cargo make run-tests          # Run tests with nextest
cargo make lint               # Run clippy with -D warnings
cargo make docs               # Generate docs with warnings as errors
cargo make spell-check        # Run typos checker
```

**Native Development:**
```bash
cargo run -p cadara           # Run the main application (opens GUI window)
cargo build --release         # Build release version
```

**WASM Development (Experimental):**
```bash
cargo make dev                # Build WASM (dev) and start server on :8080
cargo make debug              # Build WASM with debug info and serve
cargo make build-wasm-dev     # Build WASM target only (development)
cargo make test-wasm          # Run WASM-specific tests
```

WASM support is experimental and not yet complete.

### Running Single Tests

```bash
cargo nextest run -p <crate-name> <test-name>
cargo nextest run -p computegraph test_basic_node
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with the following key crates:

- **`cadara`**: Main iced application, integrates all components, runs update loop
- **`opencascade-sys`**: Low-level FFI bindings to OpenCASCADE C++ library
- **`occara`**: High-level safe Rust API wrapping OpenCASCADE for B-Rep operations
  - `geom` module: Points, Vectors, Directions, Surfaces, Curves
  - `shape` module: Edges, Wires, Faces, Solids, boolean operations (fuse, fillet, extrude, shell, etc.)
- **`computegraph`**: Core computation DAG framework with caching (powers viewport)
- **`viewport`**: Rendering and scene graph system using plugins and compute graphs
- **`module`**: Module system trait definitions for document data sections and transactions
- **`project`**: Project, document, and data management with change tracking and undo/redo
- **`workspace`**: Workspace trait defining tools and viewport plugins
- **`modeling-module`**: Concrete module implementation for parametric modeling features
- **`modeling-workspace`**: Concrete workspace for part modeling

### Core Architectural Concepts

#### ComputeGraph System

The `computegraph` crate provides a DAG-based computation framework that:
- Executes nodes in dependency order with automatic parallelization
- Caches intermediate results to avoid redundant computation
- Dynamically rebuilds the graph each frame (viewport use case)
- Supports typed and untyped node connections

Custom nodes are defined using the `#[node]` macro. The graph automatically handles type checking, caching, and execution scheduling.

#### Viewport Pipeline

The viewport uses a plugin-based architecture where **plugins are themselves ComputeGraph nodes**:
- **Viewport Plugins**: Implemented using the `#[node]` macro, added by workspaces
- Each plugin builds a scene graph (which is also a `ComputeGraph`)
- Scene graphs contain:
  - **Model nodes**: Build geometry from project data
  - **Operation nodes**: Transform data (e.g., meshing)
  - **Render nodes**: Output rendering primitives (connected to special render ports)
  - **Update nodes**: Handle input events (connected to special update ports)
- The final scene graph is executed each frame with automatic caching
- Example: `ModelingViewportPlugin` creates nodes for model → meshing → rendering pipeline

#### Module System

Modules define document data and how it can be modified:
- **Data Sections**: Four types implementing `DataSection` trait:
  - `PersistentData`: Saved to disk, tracked in undo/redo
  - `PersistentUserData`: User-specific, saved, tracked in undo/redo
  - `SessionData`: Temporary, not saved, not tracked (per-session state)
  - `SharedData`: Temporary, not saved, not tracked (synchronized between users)
- **Transactions**: Each `DataSection` defines an `Args` type and deterministic `apply(&mut self, args)` method
- Modules have unique UUIDs and are registered in `ModuleRegistry`
- Example: `ModelingModule` stores parametric modeling operations in `PersistentData`

#### Workspace System

Workspaces define how users interact with data in the application:
- **Purpose**: To "open" and work with specific types of data
- **Tools**: Provide operations that modify data (e.g., sketch tools, extrusion tools)
- **Viewport Plugins**: Define how the UI should look and how data is visualized
- **Association**: Each workspace is associated with specific module types
- Example: `ModelingWorkspace` provides tools for parametric modeling and viewport plugins for 3D visualization

#### Project System

The `project` crate provides comprehensive project management with version control:
- **Project**: Root container storing complete change history in a chronological log
- **ProjectView**: Read-only snapshot created by replaying the log
- **ChangeBuilder**: Records atomic change sets to be applied to the project
- **Documents**: Contain data sections organized hierarchically
- **Data**: Individual module instances within documents
- **Change Tracking**: Every change to persistent data is logged with:
  - User and session information
  - Branch and checkpoint support (partially implemented)
  - Complete undo/redo history (in development)
- **Workflow**: Create `ProjectView` → make changes via `ChangeBuilder` → apply to `Project` → create new view
- Designed for multi-user collaboration with offline support

#### occara (OpenCASCADE Bindings)

Provides Rust access to OpenCASCADE B-Rep kernel:
- C++ wrapper layer (parse-able subset of C++) for performance-critical code
- Rust bindings generated via `autocxx`
- Safe, idiomatic Rust API layer on top
- Error handling via C++ exception catching (not yet fully implemented)

### Important Build Details

**C++ Autocomplete in occara:**
- Run `touch cpp && bear -- cargo build` in `crates/occara/`
- Generates `compile_commands.json` for C++ language servers

## Development Guidelines

### Code Quality

- Clippy warnings are treated as errors in CI
- Use `#[warn(clippy::nursery)]` and `#[warn(clippy::pedantic)]` in crates
- Documentation warnings are errors

### Working with OpenCASCADE

- occara is incomplete - add functionality incrementally as needed
- Exceptions must be caught in C++ layer (autocxx doesn't support them)
- See `crates/occara/src/make_bottle.rs` for complete usage example

### Module Development

When creating a new module:
1. Define data structures for each of the 4 data sections (all must be `Serialize + Deserialize + Clone + Default + Debug + PartialEq`)
2. Implement `DataSection` trait for each section:
   - Define `Args` type for transactions (must be `Serialize + Deserialize + Clone + Debug + PartialEq + Hash`)
   - Implement deterministic `apply(&mut self, args: Self::Args)` method
   - Implement `undo_history_name(args: &Self::Args) -> String`
3. Implement `Module` trait with a **unique UUID** (use `uuid!()` macro)
4. Register module in `ModuleRegistry` before creating project views
5. See `crates/modeling-module` for a complete example

### Viewport Plugin Development

Viewport plugins are ComputeGraph nodes that build scene graphs:
1. Create a struct for your plugin (must be `Clone + Debug + PartialEq`)
2. Use `#[node(PluginName -> (scene, output))]` macro to implement the plugin
3. In the `run()` method:
   - Create a new `ComputeGraph`
   - Add model, operation, and render nodes
   - Connect nodes to build the pipeline
   - Return `SceneGraphBuilder` with initial state, render node, and update node ports
4. Register plugin in workspace's `viewport_plugins()` method
5. See `crates/modeling-workspace/src/viewport.rs` for complete example

### Working with ComputeGraph

When creating custom nodes:
1. Define a struct for your node (must be `Clone + Debug + PartialEq`)
2. Use `#[node(NodeName)]` or `#[node(NodeName -> output_name)]` macro
3. Implement `run(&self, param1: &Type1, param2: &Type2) -> OutputType`
4. Input parameter names become port names (e.g., `param1` → `node.input_param1()`)
5. Output types implementing `PartialEq` will be cached (use `-> !` to opt out)
6. Connect nodes with `graph.connect(from_port, to_port)`

## Key Examples & Learning Resources

**Understanding occara API:**
- `crates/occara/src/make_bottle.rs` - Complete example building a bottle with threads
  - Shows: profiles, mirroring, extrusion, filleting, fusion, shelling, lofting
  - Demonstrates high-level ergonomic API

**Understanding Project System:**
- `crates/cadara/src/lib.rs` - Main app showing full integration
  - Project creation and module registration
  - Document and data management
  - Transaction application workflow
  - Viewport integration

**Understanding Viewport Plugins:**
- `crates/modeling-workspace/src/viewport.rs` - Complete viewport plugin
  - Shows scene graph construction with ComputeGraph nodes
  - Model → meshing → rendering pipeline
  - Update and render node port wiring

**Understanding ComputeGraph:**
- `crates/computegraph/src/lib.rs` - Extensive documentation and examples
- Crate tests show various node patterns and caching behavior

## Current Implementation Status

See [README.md](README.md) for current implementation status and roadmap.

**Important:** When implementing features, verify against actual code in `crates/`, not design documents.
