# CADara (Work In Progress)

🚧 **IMPORTANT: CADara is currently in early development and not yet ready for use. Development is active but will take considerable time to reach basic functionality.** 🚧

CADara is an upcoming next-generation open-source parametric CAD software, designed with a focus on simplicity and user experience. Built on the robust [OpenCASCADE](https://dev.opencascade.org/) B-Rep kernel and modern technologies like [Rust](https://www.rust-lang.org/), [iced](https://iced.rs/), and [wgpu](https://wgpu.rs/), CADara aims to be a user-friendly open-source CAD solution while maintaining power and flexibility.

## 🌟 Key Differentiators

- **🦀 Modern Rust Architecture**: Leveraging Rust's type system for performance and reliability
- **👥 Intuitive UI/UX**: Prioritizing ease of use as a core design principle
- **🔄 Advanced Version Control**: Real-time collaboration and comprehensive history tracking
- **🔓 Fully Open-Source**: Licensed under AGPLv3 to prevent vendor lock-in

## 🎯 Project Goals

CADara's vision is both clear and ambitious:

- **👥 User-Friendly**: Redefining ease of use in open-source CAD
- **🚀 Modern Architecture**: Built with Rust's type system to prevent common CAD software bugs
- **📂 Project-Based**: Streamlined project management with linked parts and assemblies
- **🔧 Parametric Design**: Flexible design modifications using a familiar history-based approach
- **💻 Cross-Platform**: Seamless experience across desktop and web platforms
- **👥 Collaboration**: Real-time editing with CRDT-based conflict resolution, even offline
- **🔄 Version Control**: Advanced branching and merging capabilities
- **📜 Version History**: Comprehensive tracking of design evolution, allowing precise historical views
- **🔓 Open-Source**: Your designs remain yours, free from vendor lock-in

## 🆚 Current CAD Landscape

The CAD software landscape is divided between proprietary and open-source solutions, each with distinct advantages and limitations.

### Proprietary Solutions

Professional tools like SolidWorks, Fusion 360, and Onshape dominate the market, offering:

- Polished, mostly intuitive user interfaces
- Comprehensive feature sets
- Professional support and training
- Regular updates and improvements

However, they come with significant drawbacks:

- Increasing subscription costs
- Vendor lock-in through proprietary file formats
- Limited platform availability (especially on Linux)
- Risk of feature removal or pricing changes
- Data accessibility concerns if subscriptions lapse

### Existing Open-Source Alternatives

**FreeCAD**

- By far the most solid open source CAD application
- Has made significant progress with version 1.0
- Offers powerful features rivaling commercial solutions
- Large and active community
- Despite its capabilities, FreeCAD's underlying architecture can make it prone to bugs and unexpected behavior
- User experience has been a weak point, but recent initiatives are actively addressing this issue
- Complex workflows can be challenging to master

**Other Applications**

- BRL-CAD: Focus on solid modeling and ray-tracing
- CAD Sketcher: Blender-integrated parametric modeling
- CadQuery: Programming-based approach
- OpenSCAD: Script-based modeling
- Dune3D: Modern codebase with unique non-traditional workflow
- Most rely on CSG or scripting, limiting accessibility

### Core Principles

CADara bridges the gap between proprietary polish and open-source flexibility through:

1. **Modern Architecture**

   - Using Rust's type system to prevent common CAD software bugs at compile-time
   - Building on proven technologies like OpenCASCADE
   - Designing for extensibility from the ground up

2. **User Experience**

   - Focusing on intuitive workflows
   - Maintaining compatibility with familiar CAD concepts
   - Prioritizing discoverability of features

3. **Collaboration and Data Security**

   - Advanced version control with branching and merging
   - Offline-first collaboration with CRDT-based sync capabilities
   - Continuous document history and autosave, allowing precise historical views
   - Licensed under AGPLv3 to ensure continued open access

4. **Performance and Reliability**
   - Utilizing wgpu for efficient GPU acceleration across platforms (Metal on macOS, DirectX on Windows, Vulkan on Linux, WebGL/WebGPU on web)
   - Implementing efficient caching and computation strategies

## 🎨 Interface Design

CADara's interface is designed with a focus on simplicity and efficiency while maintaining professional-grade functionality. Our UI/UX follows these key principles:

- **Clean and Focused**: Minimalist design that reduces cognitive load
- **Contextual Tools**: Tools and options appear when and where you need them
- **Consistent Layout and Behavior**: Familiar patterns across all workflows
- **Discoverability through Context**: Interactive tutorials, contextual tooltips, and a context-aware help system guide users to discover relevant features and tools.

[View detailed interface mockups](docs/interface-design.md)

## 🛣️ Roadmap to Basic Usability

### Completed Components

- `computegraph`: Framework for editable computation DAGs
- Modular viewport widget with plugin system
- Module system for core components
- `occara`: High-level Rust bindings for OpenCASCADE
- WebAssembly support with C++ dependencies
- Basic project system foundation

### In Development

- Caching system for computation and viewport
- CRDT-based project system for collaboration
- Custom widget library using iced
- 3D viewport selection system
- Stable reference system in `occara`
- Constraint-based sketching
- Basic modeling module comparable to industry standards

## 🤝 Contributing

I appreciate your interest in contributing to CADara! However, as the project is still in a very early stage, I am not yet ready to accept contributions. My current focus is on establishing the core architecture and reaching a basic level of functionality first.

## 📝 Additional Information

For detailed technical documentation and architecture plans, see the [notes document](docs/Notes.md). These notes represent initially planned features and may not reflect current implementation status.
