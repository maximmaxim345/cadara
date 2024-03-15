# CADara

CADara is an upcoming next-generation open-source parametric CAD software, designed from the ground up with a focus on simplicity and user experience. Leveraging the powerful [OpenCASCADE](https://dev.opencascade.org/) B-Rep kernel and modern technologies like [Rust](https://www.rust-lang.org/), [iced](https://iced.rs/), and [wgpu](https://wgpu.rs/), CADara aims to be the most user-friendly open-source CAD solution available, while still not compromising on power and flexibility.

## 🚧 Current Status

CADara is in the initial stages of development and has not yet reached its functional stage. Keep an eye out for forthcoming updates and release announcements.

## 🎯 Project Goals

My vision for CADara is clear and ambitious. Here's what I'm building towards:

- **👥 User-Friendly**: Redefining ease of use in open-source CAD.
- **🚀 Modern Tech**: Built with modern technologies like Rust, iced, and wgpu.
- **📂 Project-Based**: Streamlined project management with linked parts and assemblies.
- **🔧 Parametric Design**: Flexibility to modify designs with a familiar history-based approach.
- **💻 Cross-Platform**: From desktop to web, CADara goes where you go.
- **👥 Collaboration**: Real-time editing and offline work with seamless CRDT-based conflict resolution.
- **🔄 Version Control**: Branch, merge, and revert with unparalleled control over your design process.
- **📜 Version History**: Navigate through your entire project's evolution with a comprehensive history of every change.
- **🔓 Open-Source**: Your designs remain yours, forever accessible and free from proprietary constraints.

This is a very ambitious set of goals, and it will take time to achieve. I'm excited to bring this vision to life, and I hope you are too!

## 🆚 Comparison to Other CAD Software

The landscape of CAD (Computer-Aided Design) software is currently marked by a stark contrast between proprietary and open-source solutions. Proprietary software like SolidWorks, Fusion 360, and Onshape lead the market with their advanced features and intuitive user interfaces but come with ever-increasing subscription costs and ecosystem lock-in due to proprietary file formats. Additionally, many proprietary tools are not available on all platforms, such as Linux, limiting the user's choice and flexibility.

On the open-source front of 3D CAD, there are a couple of notable options: FreeCAD, BRL-CAD, CAD Sketcher, CadQuery, and OpenSCAD.
Most of these options are not as capable due to the use of CSG (Constructive Solid Geometry) or are entirely script-based, making them less accessible to new users or those seeking a more intuitive user experience.
FreeCAD stands out as the most powerful alternative, offering a wide range of capabilities that approach those of its proprietary counterparts. However, despite its large feature set, capable of replacing proprietary software (especially on realthunder's branch), FreeCAD's steep learning curve, unintuitive user interface, and dated architecture make it less accessible than it could be.
Being experienced in FreeCAD, I can say that it is technically great. But the meaning of experience in the realm of FreeCAD is that you have to learn a large number of workarounds and tricks to get the software to do what you want.
Navigating the workflow in FreeCAD often feels more akin to solving a complex puzzle than engaging in a direct design process. The task of recalling which features function as intended, identifying those that do not, and locating specific operations among a sea of buttons with cryptic icons presents a considerable challenge. Achieving the desired outcome intuitively in FreeCAD often feels nearly impossible without turning to external resources such as Google searches or community forums.
This is not only a problem for beginners, but also for experienced users who want to work efficiently without constantly fighting the software.

### Redefining Open-Source CAD
While there are many efforts to improve the user experience in FreeCAD, CADara takes a different approach.
Instead of building upon the existing FreeCAD codebase, CADara is being developed from the ground up, focusing on delivering a modern and user-friendly experience that users deserve. Rather than implementing every possible feature, CADara will first prioritize the most critical features and ensure they are as intuitive as possible.
By adjusting the underlying architecture, CADara will make sure the simplest way to implement a feature is also the most user-friendly way.

By starting from scratch, CADara will not just be another CAD software; it's a reimagining of what open-source CAD can be. This clean-slate approach allows for a reevaluation of the underlying data format, ensuring that CADara is built for the modern era of design.
CADara will redefine collaboration and version control in CAD software, harnessing the full power of version control previously only available to software developers.
Whether you're working alone or in a team, online or offline, with CADara, you can forget about the fear of losing your work – the software will always have your back.
The focus on delivering a user-friendly experience and state-of-the-art collaboration tools will make CADara the most accessible and powerful open-source CAD software available.

To achieve maximal user-friendliness, CADara will aim to be workflow compatible with most popular CAD software. Instead of forcing users to relearn everything, CADara's user interface will guide users through the design process, making it easy to transition from other CAD software.
Learning software is hard, learning parametric CAD software is even harder. For this reason, the most important goal of CADara is to make the learning curve as shallow as possible.
All important actions should be easily discoverable, and the software should guide the user through the design process.
