//! # `CADara`'s Project Crate
//!
//! The `project` crate provides the core data structures and functionalities for managing projects within `CADara`.
//! It defines the [`Project`] structure, which serves as the primary container for all project-related data, including documents,
//! data sections with a comprehensive history of changes.
//!
//! ## Key Features
//!
//! - **Structured Project Representation:** Organizes project data into documents and data sections, allowing for modular
//!   and efficient management of complex data required for parametric CAD.
//! - **Data Persistence:** Supports serialization and deserialization of `Project` data, ensuring that project information
//!   can be saved and loaded reliably.
//! - **Change Tracking and Version Control:** Implements a robust change tracking system based on a chronological log of
//!   `ProjectLogEntry`. This enables features like:
//!     - Persistent undo/redo functionality.
//!     - Git-like version control with branching and merging.
//! - **Extensibility through Modules:** Allows users to define custom data types and behaviors by implementing the `module::Module`
//!   trait, providing flexibility to adapt to various project needs.
//! - **Multi-user Collaboration:** Designed to support collaborative editing by multiple users, with first-class support for
//!   offline scenarios.
//!
//! ## Core Concepts
//!
//! - **[`Project`]:** The root data structure that encapsulates all information related to a CAD project.
//! - **[`ProjectView`]:** A read-only snapshot of the `Project` at a specific point in time, generated by replaying the
//!   project's change history.
//! - **Document:** Represents a logical unit within a `Project`, containing a collection of `Data` sections.
//! - **Data:** A container for module-defined data, including persistent, user-specific, session, and shared data.
//! - **[`Module`](module::Module):** A user-defined extension that provides custom data types and associated logic, integrated into the `Project`
//!   via a [`ModuleRegistry`].
//! - **[`ChangeBuilder`]:** A mechanism for recording changes to be applied to a `Project`, ensuring atomicity and enabling
//!   undo/redo.
//!
//! ## Usage
//!
//! 1. **Create a new `Project`:**
//!    ```rust
//!    use project::Project;
//!
//!    let project = Project::new();
//!    ```
//!
//! 2. **Register Modules:**
//!    ```rust
//!    # use project::ModuleRegistry;
//!    #
//!    # // Keep this in sync with all tests in this doc.
//!    # #[derive(Debug, Clone, Default)]
//!    # struct MyModule();
//!    # #[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
//!    # struct Data(u32);
//!    # impl module::DataSection for Data {
//!    #     type Args = u32;
//!    #     fn apply(&mut self, v: u32) { self.0 = v }
//!    #     fn undo_history_name(_args: &Self::Args) -> String { String::new() }
//!    # }
//!    # impl module::Module for MyModule {
//!    #     type PersistentData = Data;
//!    #     type PersistentUserData = Data;
//!    #     type SessionData = Data;
//!    #     type SharedData = Data;
//!    #     fn name() -> String { String::new() }
//!    #     fn uuid() -> uuid::Uuid { uuid::uuid!("6ea6d750-47b9-4959-bba3-3cecc0481573") }
//!    # }
//!    let mut registry = ModuleRegistry::new();
//!    registry.register::<MyModule>();
//!    ```
//!
//! 3. **Create a `ProjectView`:**
//!    ```rust
//!    # use project::{Project, ModuleRegistry};
//!    # let project = Project::new();
//!    # let registry = ModuleRegistry::new();
//!    let project_view = project.create_view(&registry).unwrap();
//!    ```
//!
//! 4. **Modify the `Project` using a `ChangeBuilder`:**
//!    ```rust
//!    # // Keep this in sync with all tests in this doc.
//!    # #[derive(Debug, Clone, Default)]
//!    # struct MyModule();
//!    # #[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
//!    # struct Data(u32);
//!    # impl module::DataSection for Data {
//!    #     type Args = u32;
//!    #     fn apply(&mut self, v: u32) { self.0 = v }
//!    #     fn undo_history_name(_args: &Self::Args) -> String { String::new() }
//!    # }
//!    # impl module::Module for MyModule {
//!    #     type PersistentData = Data;
//!    #     type PersistentUserData = Data;
//!    #     type SessionData = Data;
//!    #     type SharedData = Data;
//!    #     fn name() -> String { String::new() }
//!    #     fn uuid() -> uuid::Uuid { uuid::uuid!("6ea6d750-47b9-4959-bba3-3cecc0481573") }
//!    # }
//!    #
//!    # use project::{Project, ModuleRegistry, ProjectView, ChangeBuilder, Path, PlannedData, PlannedDocument, DataId, DocumentId};
//!    # let mut registry = ModuleRegistry::new();
//!    # registry.register::<MyModule>();
//!    # let mut project = Project::new();
//!    let project_view = project.create_view(&registry).expect("All used modules are registered");
//!    let mut change_builder = ChangeBuilder::new();
//!
//!    // Add a new document
//!    let document: PlannedDocument = project_view.create_document(&mut change_builder, Path::try_from("/new_document".to_string()).unwrap());
//!    let data: PlannedData<MyModule> = document.create_data::<MyModule>(&mut change_builder);
//!
//!    // Get the ids of new documents and data to retrieve them later
//!    let document_id: DocumentId = *document;
//!    let data_id: DataId = *data;
//!
//!    // Apply the changes to the project
//!    project.apply_changes(change_builder, &registry).unwrap();
//!
//!    // Get a updated view with the new changes applied
//!    let project_view = project.create_view(&registry).unwrap();
//!
//!    // Apply transactions to 'data'
//!    let mut change_builder = ChangeBuilder::new();
//!    let data = project_view.open_data_by_id::<MyModule>(data_id).unwrap();
//!    data.apply_persistent(20, &mut change_builder);
//!    data.apply_session(31, &mut change_builder);
//!
//!    // Apply the changes to the project
//!    project.apply_changes(change_builder, &registry).unwrap();
//!    # // Verify that everything worked
//!    # let v = project.create_view(&registry).unwrap();
//!    # let doc = v.open_document(document_id).unwrap();
//!    # let d = doc.open_data_by_id::<MyModule>(data_id).unwrap();
//!    # assert_eq!(d.persistent.0, 20);
//!    # assert_eq!(d.session_data.0, 31);
//!    ```
//!
//! 5. **Serialize and Deserialize a `Project`:**
//!    ```rust
//!    # use project::{Project, ModuleRegistry, ProjectDeserializer};
//!    # use serde_json;
//!    # let project = Project::new();
//!    # let registry = ModuleRegistry::new();
//!    // Serialize the project
//!    let serialized_project = serde_json::to_string(&project).unwrap();
//!
//!    // Deserialize the project using ProjectDeserializer
//!    let deserializer = ProjectDeserializer { registry: &registry };
//!    let deserialized_project: Project = serde_json::from_str(&serialized_project).unwrap();
//!    ```

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

// TODO: complete refactoring of project
// - reduce registry function count by splitting data
// - implement+test: multi user, undo/redo

mod branch;
mod checkpoint;
mod data;
mod document;
mod module_data;
mod project;
mod user;

use branch::BranchId;
use checkpoint::CheckpointId;
use document::Document;
use log::{error, warn};
use module_data::{
    ErasedDataTransactionArgs, ErasedSessionData, ErasedSessionDataTransactionArgs,
    ErasedSharedDataTransactionArgs, ErasedUserDataTransactionArgs, ModuleId, MODULE_REGISTRY,
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use user::SessionId;

// Public reexports
pub use data::DataId;
pub use data::DataView;
pub use data::PlannedData;
pub use document::DocumentId;
pub use document::DocumentView;
pub use document::Path;
pub use document::PlannedDocument;
pub use module_data::ModuleRegistry;
pub use project::ProjectView;
pub use user::UserId;

/// Facilitates the deserialization of a [`Project`] from a serialized format.
///
/// [`ProjectDeserializer`] is used to reconstruct a [`Project`] instance from a serialized representation, such as JSON.
/// It leverages a [`ModuleRegistry`] to properly deserialize module-specific data types based on their uuids.
pub struct ProjectDeserializer<'a> {
    pub registry: &'a ModuleRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ProjectDeserializer<'a>
where
    'a: 'de,
{
    type Value = Project;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Put the registry in thread local storage so
        // runtime polymorphic types can be deserialized using
        // functions contained in the registry
        MODULE_REGISTRY.with(|r| {
            *r.borrow_mut() = Some(self.registry);
        });

        let project = Project::deserialize(deserializer);

        // SAFETY: Delete the registry from thread local storage to avoid
        // use after free
        MODULE_REGISTRY.with(|r| {
            *r.borrow_mut() = None;
        });
        project
    }
}

/// A single change to be applied to the [`Project`].
#[derive(Clone, Debug)]
enum PendingChange {
    Change(Change),
    SessionTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::SessionData`].
        ///
        /// The [`module::Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedSessionDataTransactionArgs,
    },
    SharedTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::SharedData`].
        ///
        /// The [`module::Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedSharedDataTransactionArgs,
    },
}

/// Like [`Path`], but also allowing the Root folder
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Deserialize, Serialize)]
pub enum FolderPath {
    Root,
    Path(Path),
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Deserialize, Serialize)]
pub enum FolderTarget {
    Root,
    Path(Path),
    OneBack,
}

/// A single change persistently applied to the [`Project`].
#[derive(Clone, Serialize, Deserialize, Debug)]
enum Change {
    CreateDocument {
        id: DocumentId,
        /// Path of the document as it will be shown to the user.
        ///
        /// In case a Document with the given `path` already exists,
        /// this `path` will automatically be renamed with [`DocumentPath::increment_name_suffix`]
        /// as many times as necessary to avoid duplicates.
        path: Path,
    },
    /// This will delete the document and all data contained in it.
    DeleteDocument(DocumentId),
    // rename the document without changing its location.
    RenameDocument {
        id: DocumentId,
        new_name: String,
    },
    // Move the document to another location, keeping its name.
    MoveDocument {
        id: DocumentId,
        new_folder: FolderPath,
    },
    // All containing folders/documents to a new location.
    MoveFolder {
        old_path: Path,
        new_path: FolderTarget,
    },
    CreateData {
        id: DataId,
        module: ModuleId,
        owner: Option<DocumentId>,
    },
    DeleteData(DataId),
    MoveData {
        id: DataId,
        new_owner: Option<DocumentId>,
    },
    Transaction {
        // Id of the Data this Transaction should be applied to.
        //
        // If `id` does not exist yet or was deleted, this Transaction will be ignored.
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::PersistentData`].
        ///
        /// The [`module::Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedDataTransactionArgs,
    },
    UserTransaction {
        // Id of the Data this Transaction should be applied to.
        //
        // If `id` does not exist yet or was deleted, this Transaction will be ignored.
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::PersistentUserData`].
        ///
        /// The [`module::Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedUserDataTransactionArgs,
    },
}

/// Entry in the log stored in [`Project`]
#[derive(Clone, Serialize, Deserialize, Debug)]
// TODO: add data for Ord
enum ProjectLogEntry {
    Changes {
        session: SessionId,
        changes: Vec<Change>,
    },
    /// Tells that the a [`Self::Changes`] before this entry (with the same [`SessionId`]) should be ignored
    Undo { session: SessionId },
    /// Tells that a [`Self::Undo`] before this entry (with the same [`SessionId`]) should be ignored
    Redo { session: SessionId },
    /// Registers a new [`SessionId`] to associate it with editing of the given user.
    ///
    /// All sessions happening on the same branch must also be registered with the same [`BranchId`].
    ///
    /// Any in [`Project::log`] used [`SessionId`] must be previously registered using this entry.
    NewSession {
        user: UserId,
        branch: BranchId,
        new_session: SessionId,
    },
    /// Add a named identifier to this position in the [`Project::log`].
    CheckPoint(CheckpointId),
}

/// Record changes to be applied to a [`Project`].
///
/// Any change that should be applied to a [`Project`] must first be recorded
/// by passing a [`ChangeBuilder`] on methods in [`ProjectView`], [`DocumentView`] or [`DataView`].
///
/// # Change Tracking
/// - **Persistent Data and User Data**: All changes are tracked and can be undone/redone
/// - **Session Data**: Changes are temporary and *not tracked* (lost on destruction of [`Project`])
/// - **Shared Data**: Changes are temporary and *not tracked* (synchronized between users)
///
/// The recorded changes are only atomic (meaining always applied together and at once) for changes to [`module::Module::PersistentData`] and [`module::Module::PersistentUserData`] on a [`DataView`],
/// changes on a [`ProjectView`] and changes on a [`DocumentView`].
/// Meaning changes to [`module::Module::SharedData`] and [`module::Module::SessionData`] will once applied using [`Project::apply_changes`] not be
/// reverted on undo.
///
/// # Features
/// This system allows for correct handling of:
/// - Undo/Redo across multiple different parts of a [`Project`]
/// - Atomic grouping of multiple changes, even in multi-user scenarios and branching/merging
/// - Complete history tracking of all changes ever applied to a [`Project`]'s persistent data
#[derive(Clone, Debug, Default)]
pub struct ChangeBuilder {
    changes: Vec<PendingChange>,
}

impl ChangeBuilder {
    /// Creates a new empty [`ChangeBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends all changes of `other` to be included in this [`ChangeBuilder`].
    ///
    /// Compared to applying multiple [`ChangeBuilder`]s separately, this will make
    /// all changes atomic. Meaning undo will revert all changes from `self` and `other` together.
    pub fn append(&mut self, mut other: Self) {
        self.changes.append(&mut other.changes);
    }
}

/// Project in the `CADara` application.
///
/// A [`Project`] describes the whole state of a `project` including:
/// - Metadata associated with the project (like name, tags)
/// - All documents contained in the project
/// - All data sections contained in the project (including [`module::Module::SessionData`] and [`module::Module::SharedData`] of all online users)
///
/// # Features
///
/// [`Project`] will support advanced features for managin CAD projects, including:
/// - Persistent undo/redo, even after restarts
/// - A git like version control system, allowing branching, merging and rebasing
/// - Support of storing any user required data by implementing [`module::Module`]
/// - Complete storage of the complete history of a [`Project`]
/// - Multi user collaborative editing including first in class offline support.
///
/// # Viewing the Project
///
/// Data contained in a [`Project`] can only be viewed through a [`ProjectView`] by using [`Project::create_view`].
///
/// # Making Changes
///
/// To make and save changes to a [`Project`], use a [`ChangeBuilder`] to record changes, then apply
/// them using [`Project::apply_changes`].
///
/// # Serialization and Deserialization
///
/// Serialize a [`Project`] like any other type implementing [`serde::Deserialize`],
/// this will not save any shared and session data.
///
/// To deserialize, you must use [`ProjectDeserializer`] with a [`ModuleRegistry`] with all containing [`module::Module`]s registered.
/// While [`Project`] implements [`serde::Serialize`], it will error on any non trivial project
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Project {
    /// The user owning this [`Project`] struct.
    ///
    /// By default, this is the `local` user.
    user: UserId,
    /// Session that will be used for new changes.
    ///
    /// Must be reset in case this [`Project`] is sent to another device to avoid
    /// undo/redo conflicts.
    session: Option<SessionId>,
    /// The id of the branch, this [`Project`] is representing.
    ///
    /// By default, this is `main`.
    branch: BranchId,
    /// Chronological list of entries required to rebuild the entire [`ProjectView`] excluding
    /// shared and session data
    log: Vec<ProjectLogEntry>,
    /// [`HashMap`] with all [`module::Module::SharedData`] (of this user) in this project.
    #[serde(skip)]
    shared_data: HashMap<DataId, module_data::ErasedSharedData>,
    /// [`HashMap`] with all [`module::Module::SessionData`] in this project.
    #[serde(skip)]
    session_data: HashMap<DataId, ErasedSessionData>,
}

/// Errors that can occur when creating a project view
#[derive(thiserror::Error, Debug)]
pub enum ProjectViewError {
    /// Attempted to load data for a module that isn't registered
    #[error("The module {0} is used, but not registered in the registry")]
    UnknownModule(ModuleId),
}

/// Errors that occur
#[derive(thiserror::Error, Debug)]
pub enum ApplyError {
    #[error("The module {0} is used, but not registered in the registry")]
    UnknownModule(ModuleId),
    #[error("ModuleMismatch")]
    ModuleMismatch,
}

impl Project {
    /// Creates a new [`ProjectView`] by replaying the project's change history.
    ///
    /// # Description
    /// Reconstructs the current state of the project by applying all logged changes
    /// in chronological order, creating a consistent view of:
    /// - All documents and their metadata
    /// - All persistent data associated with modules
    /// - Current document structure and relationships
    ///
    /// # Arguments
    /// * `reg` - The [`ModuleRegistry`] containing all module implementations that were
    ///           ever used in the project
    ///
    /// # Returns
    /// Returns a [`ProjectView`] representing the current state of the project.
    ///
    /// # Errors
    /// Returns an error if:
    /// - A required module is not found in the registry
    #[expect(clippy::too_many_lines)]
    #[expect(clippy::cognitive_complexity)]
    pub fn create_view(&self, reg: &ModuleRegistry) -> Result<ProjectView, ProjectViewError> {
        let mut data = HashMap::new();
        let mut documents = HashMap::new();
        let mut sessions = HashMap::new();

        for log_entry in &self.log {
            match log_entry {
                ProjectLogEntry::Changes { session, changes } => {
                    for change in changes {
                        match change {
                            Change::CreateDocument { id, path: _ } => {
                                documents.insert(*id, Document::default());
                            }
                            Change::DeleteDocument(document_id) => {
                                if let Some((_, document)) = documents.remove_entry(document_id) {
                                    for data_id in &document.data {
                                        data.remove(data_id);
                                    }
                                }
                            }
                            Change::RenameDocument { id: _, new_name: _ }
                            | Change::MoveDocument {
                                id: _,
                                new_folder: _,
                            }
                            | Change::MoveFolder {
                                old_path: _,
                                new_path: _,
                            } => {
                                // TODO: implement this
                            }
                            Change::CreateData { id, module, owner } => {
                                data.insert(
                                    *id,
                                    (reg.0
                                        .get(module)
                                        .ok_or(ProjectViewError::UnknownModule(*module))?
                                        .init_data)(),
                                );
                                if let Some(owner) = owner {
                                    documents
                                        .entry(*owner)
                                        .or_insert_with(Default::default)
                                        .data
                                        .push(*id);
                                }
                            }
                            Change::DeleteData(id) => {
                                data.remove(id);
                            }
                            Change::MoveData { id, new_owner } => {
                                for document in documents.values_mut() {
                                    document.data.retain(|data_id| data_id != id);
                                }
                                if let Some(new_owner) = new_owner {
                                    if let Some(doc) = documents.get_mut(new_owner) {
                                        doc.data.push(*id);
                                    } else {
                                        error!("Can not move data to non existent document");
                                    }
                                }
                            }
                            Change::Transaction { id, args } => {
                                let reg = reg
                                    .0
                                    .get(&args.module)
                                    .ok_or(ProjectViewError::UnknownModule(args.module))?;
                                match data.get_mut(id) {
                                    Some(data) if data.module == args.module => {
                                        if let Err(err) = (reg.apply_data_transaction)(data, args) {
                                            error!("Failed to apply Transaction: {}", err);
                                        }
                                    }
                                    Some(_) => {
                                        error!(
                                            "Data and DataArgs of {id} do not have the same Module type"
                                        );
                                    }
                                    None => {}
                                }
                            }
                            Change::UserTransaction { id, args } => {
                                if let Some(user) = sessions.get(session) {
                                    if self.user == *user {
                                        let reg = reg
                                            .0
                                            .get(&args.module)
                                            .ok_or(ProjectViewError::UnknownModule(args.module))?;
                                        match data.get_mut(id) {
                                            Some(data) if data.module == args.module => {
                                                if let Err(err) =
                                                    (reg.apply_user_data_transaction)(data, args)
                                                {
                                                    error!("Failed to apply Transaction: {}", err);
                                                }
                                            }
                                            Some(_) => {
                                                error!("UserData and UserDataArgs of {id} do not have the same Module type");
                                            }
                                            None => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ProjectLogEntry::Undo { session: _ } => todo!("undo/redo is not supported yet"),
                ProjectLogEntry::Redo { session: _ } => todo!("undo/redo is not supported yet"),
                ProjectLogEntry::NewSession {
                    user,
                    new_session,
                    branch: _,
                } => {
                    sessions.insert(*new_session, *user);
                }
                ProjectLogEntry::CheckPoint(_) => {}
            };
        }

        for (id, session_data) in &self.session_data {
            match data.get_mut(id) {
                Some(data) if data.module == session_data.module => {
                    if let Err(err) =
                        (reg.0
                            .get(&session_data.module)
                            .ok_or(ProjectViewError::UnknownModule(session_data.module))?
                            .replace_session_data)(data, session_data)
                    {
                        error!("Failed to replace Data::session with SessionData: {err}");
                    }
                }
                Some(_) => {
                    error!("SessionData and Data of {id} do not have the same Module type");
                }
                None => {}
            }
        }

        for (id, shared_data) in &self.shared_data {
            match data.get_mut(id) {
                Some(data) if data.module == shared_data.module => {
                    if let Err(err) =
                        (reg.0
                            .get(&shared_data.module)
                            .ok_or(ProjectViewError::UnknownModule(shared_data.module))?
                            .replace_shared_data)(data, shared_data)
                    {
                        error!("Failed to replace Data::shared with SharedData: {err}");
                    }
                }
                Some(_) => {
                    error!("SharedData and Data of {id} do not have the same Module type");
                }
                None => {}
            }
        }

        Ok(ProjectView {
            user: UserId::local(),
            data,
            documents,
        })
    }

    /// Creates a new empty project
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the changes recorded using the [`ChangeBuilder`] to this project.
    ///
    /// After applying the changes, use [`Project::create_view`] to see the new state
    /// of the [`Project`].
    ///
    /// # Errors
    /// Returns an error if:
    /// - A required module is not found in the registry
    /// - Transaction assumes an incorrect Module
    ///   (should normally never happen)
    #[expect(clippy::missing_panics_doc, reason = "expects are prechecked")]
    pub fn apply_changes(
        &mut self,
        cb: ChangeBuilder,
        reg: &ModuleRegistry,
    ) -> Result<(), ApplyError> {
        let session = *self.session.get_or_insert_with(|| {
            let new_session = SessionId::new();
            self.log.push(ProjectLogEntry::NewSession {
                user: self.user,
                branch: self.branch,
                new_session,
            });
            new_session
        });

        // Verify all changes first
        for change in &cb.changes {
            match change {
                PendingChange::Change(_) => {
                    // This targets persistent data, so if this crates usage of ChangeBuilder
                    // are correct and we assume Uuids are unique, this is already correct.
                    // If in case for some reason this is not correct, this crate can still handle malformed Projects.
                }
                PendingChange::SessionTransaction { id, args } => {
                    if let Some(data) = self.session_data.get(id) {
                        if data.module != args.module {
                            return Err(ApplyError::ModuleMismatch);
                        }
                        if !reg.0.contains_key(&args.module) {
                            return Err(ApplyError::UnknownModule(args.module));
                        }
                    } else {
                        // We will create the data in the next step
                    }
                }
                PendingChange::SharedTransaction { id, args } => {
                    if let Some(data) = self.shared_data.get(id) {
                        if data.module != args.module {
                            return Err(ApplyError::ModuleMismatch);
                        }
                        if !reg.0.contains_key(&args.module) {
                            return Err(ApplyError::UnknownModule(args.module));
                        }
                    } else {
                        // We will create the data in the next step
                    }
                }
            }
        }

        // Now apply the changes
        let changes = cb
            .changes
            .into_iter()
            .filter_map(|change| match change {
                PendingChange::Change(change) => Some(change),
                PendingChange::SessionTransaction { id, args } => {
                    let reg = reg.0.get(&args.module).expect("already checked above");
                    let data = self
                        .session_data
                        .entry(id)
                        .or_insert_with(|| (reg.init_session_data)());
                    if let Err(err) = (reg.apply_session_data_transaction)(data, &args) {
                        error!("Failed to apply SessionData Transaction: {}", err);
                    }
                    None
                }
                PendingChange::SharedTransaction { id, args } => {
                    let reg = reg.0.get(&args.module).expect("already checked above");
                    let data = self
                        .shared_data
                        .entry(id)
                        .or_insert_with(|| (reg.init_shared_data)());
                    if let Err(err) = (reg.apply_shared_data_transaction)(data, &args) {
                        error!("Failed to apply SharedData Transaction: {}", err);
                    }
                    None
                }
            })
            .collect();

        self.log.push(ProjectLogEntry::Changes { session, changes });
        Ok(())
    }
}
