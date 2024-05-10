use crate::{user::User, Project};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};
use uuid::Uuid;

/// Errors that can occur when interacting with the `ProjectManager`.
///
/// This enum provides a robust way of handling errors that may occur when
/// performing operations with the `ProjectManager`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagerError {
    /// The user lacks the necessary permissions for the operation.
    PermissionDenied,
    /// The requested project could not be found.
    NotFound,
}

/// The location of a managed project within the `CADara` application.
///
/// Managed projects are stored in a location controlled by `CADara`.
/// This location can be either a local directory or a remote host.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ManagedProjectLocation {
    /// The host where the project is located.
    pub host: String,
    /// The unique identifier of the project.
    pub uuid: Uuid,
}

/// Possible locations of a project within the `CADara` application.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProjectLocation {
    /// The project is located in a `CADara`-managed location, either on a host or locally.
    Managed(ManagedProjectLocation),
    /// The project is located in a user-specified location on the user's machine.
    Local(PathBuf),
}

/// Possible locations for creating a new project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectCreationLocation {
    /// The new project will be managed by `CADara` and located on a host.
    Managed(String),
    /// The new project will be located locally on the user's machine.
    Local(PathBuf),
}

#[derive(Clone, Default, Debug)]
struct InternalProjectManager {
    projects: HashMap<ProjectLocation, Project>,
}

/// Manages the lifecycle of projects within the `CADara` application.
///
/// The `ProjectManager` consolidates multiple instances of the same project into a single instance.
/// This ensures that all changes are synchronized across all instances of the project and
/// prevents data corruption.
///
/// Projects can be accessed/created in two ways:
/// 1. By specifying the project's location.
/// 2. By using a managed project location.
///
/// Managed project locations are controlled by the `ProjectManager` and can be
/// either on a remote host or locally. This is the recommended way to access/create
/// projects, as it better supports multi-user environments and simplifies project creation and
/// management for the user.
///
/// # Warning
///
/// Do not share `Project`s between multiple instances of the `ProjectManager` in the same or different applications.
/// Opening the same (local) project with different `ProjectManager` instances multiple times can lead to data corruption.
#[derive(Clone, Default, Debug)]
pub struct ProjectManager {
    manager: Rc<RefCell<InternalProjectManager>>,
}

impl ProjectManager {
    /// Creates a new `ProjectManager`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a project at the specified location for the specified user.
    ///
    /// If the same project is opened multiple times, the `ProjectManager`
    /// consolidates all instances into a single instance, synchronizing data across all instances.
    ///
    /// # Returns
    ///
    /// The opened project.
    ///
    /// # Errors
    ///
    /// If the user lacks permission to open the project or if the project does not exist, an error is returned.
    pub fn open(&self, location: ProjectLocation, user: User) -> Result<Project, ManagerError> {
        let mut manager = self.manager.borrow_mut();
        // TODO: this is a temporary hack until serialization is implemented
        let project = manager
            .projects
            .entry(location)
            .or_insert_with(|| Project::new_with_path(String::new(), user, PathBuf::new()));
        Ok(project.clone())
    }

    /// Creates a new project at the specified location for the specified user.
    ///
    /// # Returns
    ///
    /// The location of the created project. Use this location to open the project with `open`.
    ///
    /// # Errors
    ///
    /// If the user lacks permission to create a project at the specified location or if the project already exists, an error is returned.
    pub fn create(_user: User) -> Result<ProjectLocation, ManagerError> {
        todo!()
    }
}
