#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use computegraph::DynamicNode;

pub struct Action();

pub struct Tool {
    _name: String,
    _action: Action,
}

/// Represents a group of tools in `CADara`
pub struct Toolgroup {
    _name: String,
    _tools: Vec<Tool>,
}

/// Represents a workspace in `CADara`
///
/// A workspace is a self-contained environment where users can perform specific tasks.
/// Examples of workspaces include Design and Assemble.
pub trait Workspace {
    /// Returns all tools which can be invoked in this workspace.
    ///
    /// The tools are grouped into `Toolgroup`s, based on their functionality.
    /// For example, all tools related to creating a new part should be grouped together,
    /// while all tools related to creating constraints should be in a separate group.
    fn tools(&self) -> Vec<Toolgroup>;

    /// Returns a vector of `DynamicNode`s representing viewport plugins for this workspace.
    ///
    /// These plugins can modify or extend the viewport's functionality to enhance user experience.
    /// The system will use the first compatible plugin from the returned vector.
    ///
    /// # Returns
    ///
    /// - A `Vec<DynamicNode>` containing viewport plugins.
    /// - An empty vector if no plugins are required for this workspace
    ///
    /// # Note
    ///
    /// Each `DynamicNode` should represent a viewport plugin implementation.
    fn viewport_plugins(&self) -> Vec<DynamicNode>;
}
