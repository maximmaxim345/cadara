# CADara (Work In Progress)

🚧 **IMPORTANT: CADara is currently in the very early stages of development and is not yet usable. It will likely remain in this state for a considerable time.** 🚧

CADara is an upcoming next-generation open-source parametric CAD software, designed from the ground up with a focus on simplicity and user experience. Leveraging the powerful [OpenCASCADE](https://dev.opencascade.org/) B-Rep kernel and modern technologies like [Rust](https://www.rust-lang.org/), [iced](https://iced.rs/), and [wgpu](https://wgpu.rs/), CADara aims to be the most user-friendly open-source CAD solution available, while still not compromising on power and flexibility.

## 🎯 Project Goals

The vision for CADara is clear and ambitious. Here's what this project aims to achieve:

- **👥 User-Friendly**: Redefining ease of use in open-source CAD.
- **🚀 Modern Tech**: Built with modern technologies like Rust, iced, and wgpu.
- **📂 Project-Based**: Streamlined project management with linked parts and assemblies.
- **🔧 Parametric Design**: Flexibility to modify designs with a familiar history-based approach.
- **💻 Cross-Platform**: From desktop to web, CADara goes where you go.
- **👥 Collaboration**: Real-time editing and offline work with seamless CRDT-based conflict resolution.
- **🔄 Version Control**: Branch, merge, and revert with unparalleled control over your design process.
- **📜 Version History**: Navigate through your entire project's evolution with a comprehensive history of every change.
- **🔓 Open-Source**: Your designs remain yours, forever accessible and free from proprietary constraints.

Please note that this is a very ambitious set of goals, and it will take a significant amount of time and effort to achieve. The project will take a considerable amount of time to even reach a basic level of functionality, so please be patient.

## 🆚 Comparison to Other CAD Software

The landscape of Computer-Aided Design (CAD) software is currently divided between proprietary and open-source solutions. Proprietary software like SolidWorks, Fusion 360, and Onshape dominate the market with their advanced features and intuitive user interfaces. However, they come with ever-increasing subscription costs and ecosystem lock-in due to proprietary file formats. Additionally, many proprietary tools are not available on all platforms, such as Linux, limiting users' choice and flexibility.

In the open-source 3D CAD realm, notable options include FreeCAD, BRL-CAD, CAD Sketcher, CadQuery, and OpenSCAD. Most of these alternatives are less capable due to their reliance on Constructive Solid Geometry (CSG) or are entirely script-based, making them less accessible to new users or those seeking a more intuitive user experience.

FreeCAD stands out as the most powerful open-source alternative, offering a wide range of capabilities that approach those of its proprietary counterparts. However, despite its extensive feature set, FreeCAD's steep learning curve, unintuitive user interface, and dated architecture make it less accessible than it could be. As an experienced FreeCAD user, I can attest to its technical prowess, but mastering FreeCAD requires learning numerous workarounds and tricks to achieve desired results.

Navigating FreeCAD's workflow often feels like solving a complex puzzle rather than engaging in a straightforward design process. Users must remember which features function as intended, identify those that don't, and locate specific operations among a sea of buttons with cryptic icons. Achieving the desired outcome intuitively in FreeCAD often feels nearly impossible without resorting to external resources like online searches or community forums. This challenge affects not only beginners but also experienced users who want to work efficiently without constantly fighting the software.

### Redefining Open-Source CAD

While there are many efforts to improve the user experience in FreeCAD, CADara takes a different approach. Instead of building upon the existing FreeCAD codebase, CADara is being developed from the ground up, focusing on delivering a modern and user-friendly experience that users deserve. Rather than implementing every possible feature, CADara will prioritize the most critical features and ensure they are as intuitive as possible. By optimizing the underlying architecture, CADara will ensure that the simplest way to implement a feature is also the most user-friendly way.

Starting from scratch allows CADara to reimagine what open-source CAD can be. This clean-slate approach enables a reevaluation of the underlying data format, ensuring that CADara is built for the modern era of design. CADara will revolutionize collaboration and version control in CAD software, harnessing the full power of version control previously only available to software developers. Whether working alone or in a team, online or offline, CADara users can forget about the fear of losing their work – the software will always have their back. The focus on delivering a user-friendly experience and cutting-edge collaboration tools will make CADara the most accessible and powerful open-source CAD software available.

To achieve maximum user-friendliness, CADara will aim to be workflow compatible with many popular CAD applications. Instead of forcing users to relearn everything, CADara's intuitive user interface will guide users through the design process, making it easy to transition from other CAD software. Learning software is challenging, and learning parametric CAD software is even more demanding. For this reason, CADara's primary goal is to make the learning curve as gentle as possible. All essential actions should be easily discoverable, and the software should provide clear guidance to users throughout the design process, ensuring a smooth and enjoyable experience for both novice and experienced designers alike.
