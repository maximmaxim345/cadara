Notes and Ideas for CADara
====

This document should be viewed as my notes for a rough outline of the final application and may be out of date.

# Workflow
Parts and Assemblies are stored inside Projects, which are managed by CADara. This will later allow us to create better handling of changed dependencies of Assemblies (and maybe true collaborative editing).

## Overview
To display, search, and create all parts of the current Project, the Overview can be used. Here the user can also create a new Project and manage all Projects of the user. This will also be the page which is first shown on opening the Application. Here a tutorial or manual can be added to help new users.

## Workspaces
Workspaces are used to separate different tasks.

### Design
Here the user can create sketches, extrude, make fillets, and more. The history of those actions can also be viewed and modified. Section Views can be created, whose state is not shared by multiple users, which can still show the whole model with transparency (if requested).

### Assemble
Multiple links to parts can be brought in from the project and positioned using constraints.

## History
The History can be displayed either horizontally or vertically, according to the user's preference.

# Tools
All tools are actions that change the document. Similar tools are grouped into groups with a title. All Toolgroups of the current Workspace are displayed in a horizontal bar above the 3D Viewport. Frequent tools are directly visible, while all other tools (which don't fit on screen) can be viewed by expanding the group.

The activation of a tool can show a menu where the user can set parameters before committing to the change. The user's selection prior to the invocation should also be used as the default value. A responsive preview will display (approximately) how the model would look after committing the action.

# Implementation
This Application is written using Rust, iced-rs, and OpenCascade. If the need arises, we can include rend3 as a powerful rendering library. All graphics are done through wgpu to ensure high performance, cross-platform support, and even potential future web support.

## Document
Each Document is inside a single project and contains data managed by modules. The complete state of an open document is represented by all storage sections of all included modules.

TODO: Data is saved in a project; a project consists of documents
TODO: A document has only one corresponding module

A module can use the following storage sections:
- Persistent Data
    - This is stored on disk
- Persistent User Data
    - User-specific options that are worth remembering (like viewing options)
    - If a new user opens the document, this section will be empty
    - This is stored on disk
- User State
    - This data is not saved persistently and serves for storing temporary data for the current user
- Shared State
   - This data behaves like User State but is synchronized between all users who are currently editing the document

## Transient States

Transient states refer to the temporary and often dynamic modifications that have not yet been committed to the document. These include real-time user actions such as dragging to extrude a surface, adjusting constraints, sketching new elements, or creating a new extrusion. Managing these states is crucial for supporting a responsive and collaborative design process where multiple users may interact with the same elements concurrently. This is, for example, used in sketches where moving a line just generates the transient that a point is moved by a vector. The Model generator can then solve the correct placement of the geometry. Each transient is associated with a module. To visualize the transient, it can register a specific Viewport Plugin as described in the Viewport section.

## Tools

Tools define all user-runnable actions that modify the document. The UI is responsible for populating a transient type according to the user's needs. An instance of this type has the same lifetime as the transient corresponding to it and is used for creating a new action or editing an existing one. The transient will be handled by the specific module the tool operates in. It is also possible that invoking a tool is instantaneous. Here the transient is immediately applied.

- **UI Integration**: The tool's UI seamlessly integrates with the application's interface through the use of a stable API.
- **Parameter Adjustment**: The parameters are linked in real-time to visual updates in the viewport, providing immediate feedback.
- **Input Validation**: Incorporates mechanisms to validate user input and prevent invalid operations that could cause errors or unexpected results. Offers corrective suggestions or constrained input fields where applicable.

## Modules

Modules add core functionality to this Application. Examples of such modules are Design for constructing new parts and Assemble to define constraints between multiple parts. A single document is allowed to incorporate multiple modules and even multiple instances of the same module. A powerful linking system is in place to attach specific data from any section to any other data. This system also allows links to be made across different modules in the same document or even other documents in the same project.

### Basic Structure of a Module

A module is responsible for defining the following behavior:
- **Save Data**: Each module has a clearly defined structure of data it can store in each storage section.
- **Builder Function**: This function should build the saved data, for example, the final BREP model.
- **Transient Handler**: This part is responsible for applying transients to the underlying data and handling conflicts.

## The Design Module

The Model Builder will be responsible for generating a BREP model from a list of features. This will be implemented with a functional paradigm (with caching) to ensure coherence between the saved and displayed model. Each feature is built sequentially:
- The specific builder function of the feature receives:
    - The model at the previous state (when the first feature, this will be an empty model)
    - Global building options
        - Like quality, calculation precision
    - A strict API to query any other required data to compute the feature
        - This allows us to implicitly generate a list of dependencies for caching purposes
- And produces the model after applying the action
To accommodate visual feedback through the GUI, a builder can attach properties to faces, edges, solids, or the whole model.

### Features

Features are implemented as Tools, with the following additional members required by the Part Module.

#### Data
A tool will have a statically known data type that includes everything to allow the building of this feature. This data will be passed to the builder function when computing the BREP model. Two instances of data should be able to be diffed to allow conflict resolution.

#### Builder Function
The builder function is specific to each implemented tool and is executed by the Part Builder.

#### Migrator
This function adapts existing designs to new versions of the tool, ensuring legacy support and data integrity.

#### Internal API
- Each tool may expose an internal API for scripting or automation, permitting advanced users to programmatically drive the tool's operation.
- This API should be well-documented and stable across versions, providing a reliable interface for developers.
- Stability can potentially also be realized by using an old version of the API, which generates an old version of Data, which then can be migrated to the installed version of the tool: stability could be automatically guaranteed.

#### Conflict Resolver

A Feature can optionally define a conflict-resolving strategy if two users simultaneously edit the same feature.
TODO: Rewrite to: can define custom modify transient

## Workspaces
Changing into a workspace can require arguments. These arguments are passed down to all tools contained in it. This is especially useful for complex workspaces like Sketch, where the argument is the specific sketch being edited. Activation of workspaces is saved in a stack; leaving it returns the user to the previous one. A workspace can define a Viewport Plugin, as described in the Viewport section.

## Viewport

The Viewport is the central UI component managing and visualizing documents in the Application. It coordinates plugins for rendering, interaction, and visualization while maintaining the hierarchical scene graph. To add functionality to the Viewport, Viewport Plugins and Extensions can be used. The scene graph will be rebuilt each frame, but caching is used to reduce computational load.

### Viewport Plugins

Viewport Plugins define the core functionality of the Viewport and can only be included by Workspaces in two ways:

1. Add
    - The Viewport plugin will be added as the last Viewport plugin on the Viewport plugin stack.
    - This can change the behavior of the viewport while keeping some behavior of previous workspaces.
2. Replace
    - All previously added Viewport plugins will be replaced by the designated plugin.
    - This is used when editing a Technical drawing; the previous 3D Viewport is not required.

Each plugin is run sequentially and is given full R/W access to the scene graph.

### Viewport Extensions

Viewport Extensions can modify the scene graph within the confines of nodes designated to them and are run after all Viewport Plugins. The order of execution is always top-down from the root node. Extensions are allowed to:
- Query nodes immediately beneath itself
- Replace itself with another node (or a subgraph of nodes)

### Transient State Visualization

To ease handling of transient states, a handler library can be used.

It should be executed in a Viewport Plugin after its Scene Graph has been built. The handler is given R/W access to modify the graph to give specific visualization viewport plugins scoped access to the graph.

This clean separation of concerns balances simplicity for those viewport plugins while empowering enough complexity required for rich visualization.

### Scene Graph Nodes

The Scene Graph consists of a hierarchy of nodes that collectively store all necessary information required to render the viewport. When required, the scene graph can also be used independently of the viewport. Cycles are not allowed. Nodes can have 0..n input connections and 0..m output connections (0 in the case of render nodes).

The core Node types are:

**Data Nodes**:
- Stores a link to data:
- If the data needs to be built first, the corresponding module builder is called.
- If the referenced data is in the document struct (all sections are allowed), it will be directly passed (with read permissions).

**Subgraph Nodes**:
- Subgraph, treated as one node
- Organize graph hierarchy
- Attached extensions can access the whole graph

**Operation Nodes**:
- Apply operation on the input data, resulting in the output

**Render Nodes**:
- Pass the rendering data to the rendering stage, with the designated render pipeline

**Each node additionally has the following attributes**:
- Target extensions section describing what extensions can be applied to this node
    - List of identifiers for viewport extensions
        - Each identifier targets a specific viewport extension
        - Are executed chronologically, but extensions are allowed to modify this list
    - Includes data, readable by those extensions
- Section where each node can save application-specific data

This subdivision of labor between node roles allows building up complex CAD models with associated behaviors using declarative relationships. Nodes can be reused, extended, or overridden to enable customization. Since the Scene Graph's nodes do not directly contain data required for rendering, the scene graph itself can be built declaratively without a large performance hit.

### Running the Scene Graph

To render the Viewport, a depth-first traversal of the Scene Graph is performed. Each node is executed in the order visited, with node inputs populated from outputs of previously visited nodes.

If a node triggers an asynchronous operation, it registers a callback and immediately outputs a special "pending" token instead of its typical output. Downstream nodes detect this pending state, skip their actual work, but generate placeholder outputs by preserving prior valid state.

This pending status propagates through the graph transitively. When the original async node callback fires, it publishes the actual completed data via a graph reset event to downstream consumers, ensuring synchronization.

The graph traversal maintains a pool of pending async operations. If an operation remains pending longer than a threshold duration, a warning is surfaced to the user.

When resetting from a pending state, short animation transitions are used to smoothly introduce the new data and prevent visual popping. Incremental pending scene updates are also supported when possible.

Cyclic links in the graph are detected and cause an error. Render nodes are always re-executed while intermediate operation nodes are cached - the node's compute function is only re-run when inputs have changed since the last execution.

An initial traversal computes all non-async nodes. Subsequent traversals reuse cached results where possible before executing render nodes to produce frame output. Async nodes trigger pending states until their callback data is integrated via reset events.

This leverages caching to enable smooth animation and interaction even with heavy computational CAD workloads. The pending mechanism crucially prevents flickering or intermediate scene errors. Performance is tuned by the polling duration, transition animations, and incremental update support for specific asynchronous operations.

## Selection System

Selections are handled by a handler API. Selections are primarily implemented by using Transients, which already have a visualization system. The API provides methods for identifying selectable elements, indicating active selections, managing selection sets, and detecting conflicts. Dedicated plugins create custom visual treatments on top to reflect highlighted entities.
