//! Data and dependency management for Projects.
//!
//! [`Project`]s are the primary data structure within `CADara`. Encapsulating documents and data sections,
//! storing everything (except very short lived data in the `viewport`).
//!
//! This module provides functionality to create, open, modify, and save projects.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

// TODO: complete refactoring of project
// - refactor user.rs, including Session support
// - add session support
// - add support for data sections other than persistent
// - adjust module to disallow errors
// - allow for errors in project, in create view and deserialization
// - add metadata, not only do documents, but also to projects and data
// - support undo/redo
// - reenable tests
// - update rest of codebase
// - Design Task (branch) > Changes (checkpoint) > Actions (change -> action, changes -> actions)
// - Revisions use CheckPoint, but also make a immutable ProjectArchive w.o redundant data

mod data;
mod document;
mod module_data;
mod project;
mod user;

use document::Document;
use module_data::{
    ErasedData, ErasedSessionData, ErasedTransactionArgs, ModuleId, ModuleRegistry, MODULE_REGISTRY,
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use user::User;
use uuid::Uuid;

// Public reexports
pub use data::DataId;
pub use data::DataView;
pub use document::DocumentId;
pub use document::DocumentView;
pub use project::ProjectView;

/// Helper to deserialize a [`Project`].
///
/// Use this to deserialize a [`Project`]. The passed [`ModuleRegistry`] needs to contain all [`Module`]s
/// contained in the [`Project`].
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

/// A single change persistently applied to the [`Project`]
#[derive(Clone, Serialize, Deserialize, Debug)]
enum Change {
    CreateDocument {
        id: DocumentId,
        // TODO: add metadata, not only do documents, but also to projects and data
        // name: String,
    },
    DeleteDocument(DocumentId),
    // RenameDocument {
    //     id: DocumentId,
    //     new_name: String,
    // },
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
        id: DataId,
        /// Stores the [`TransactionArgs`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedTransactionArgs,
    },
    UserTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedTransactionArgs,
    },
}

/// Entry in the log stored in [`Project`]
#[derive(Clone, Serialize, Deserialize, Debug)]
enum ProjectLogEntry {
    Changes {
        session: Uuid,
        changes: Vec<Change>,
    },
    /// Tells that the a [`Self::Changes`] before this entry (with the same session id) should be ignored
    Undo {
        session: Uuid,
    },
    /// Tells that a [`Self::Undo`] before this entry (with the same session id) should be ignored
    Redo {
        session: Uuid,
    },
    /// Registers a new [`Session`] to associate it with the given [`User`]
    NewSession {
        user: User,
        session: Uuid,
    },
}

/// Record changes to be applied to a [`Project`]
///
/// Any change that should be applied to a [`Project`] must first be recorded
/// by passing a [`ChangeBuilder`] on methods in [`ProjectView`], [`DocumentView`] or [`DataView`].
///
/// # Change Tracking
/// - **Persistent Data and User Data**: All changes are tracked and can be undone/redone
/// - **Session Data**: Changes are temporary and *not tracked* (lost on destruction of [`Project`])
/// - **Shared Data**: Changes are temporary and *not tracked* (synchronized between users)
///
/// The recorded changes are only atomic (meaining always applied together and at once) for changes to [`Module::PersistentData`] and [`Module::PersistentUserData`] on a [`DataView`],
/// changes on a [`ProjectView`] and changes on a [`DocumentView`].
/// Meaning changes to [`Module::SharedData`] and [`Module::SessionData`] will once applied using [`Project::apply_changes`] not be
/// reverted on undo.
///
/// # Features
/// This system allows for correct handling of:
/// - Undo/Redo across multiple different parts of a [`Project`]
/// - Atomic grouping of multiple changes, even in multi-user scenarios and branching/merging
/// - Complete history tracking of all changes ever applied to a [`Project`]'s persistent data
#[derive(Clone, Debug, Default)]
pub struct ChangeBuilder {
    changes: Vec<Change>,
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
/// - All data sections contained in the project (including [`Module::SessionData`] and [`Module::SharedData`] of all online users)
///
/// # Features
///
/// [`Project`] will support advanced features for managin CAD projects, including:
/// - Persistent undo/redo, even after restarts
/// - A git like version control system, allowing branching, merging and rebasing
/// - Support of storing any user required data by implementing [`Module`]
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
/// To deserialize, you must use [`ProjectDeserializer`] with a [`ModuleRegistry`] with all containing [`Module`]s registered.
/// While [`Project`] implements [`serde::Serialize`], it will error on any non trivial project
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Project {
    /// Chronological list of entries required to rebuild the entire [`ProjectView`] excluding
    /// shared and session data
    log: Vec<ProjectLogEntry>,
    /// [`HashMap`] with all [`module::Module::SharedData`] (of this user) in this project.
    #[serde(skip)]
    #[expect(dead_code)]
    shared_data: HashMap<DataId, module_data::ErasedSharedData>,
    /// [`HashMap`] with all [`module::Module::SessionData`] in this project.
    #[serde(skip)]
    #[expect(dead_code)]
    session_data: HashMap<DataId, ErasedSessionData>,
}

/// Errors that can occur when creating a project view
#[derive(thiserror::Error, Debug)]
pub enum ProjectViewError {
    /// Attempted to load data for a module that isn't registered
    #[error("The module {0} is required, but not registered in the registry")]
    UnknownModule(ModuleId),
    /// The [`Project`] is malformed or corrupted
    // TODO: siltently handle this error
    #[error("The project is malformed")]
    InvalidProject,
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
    pub fn create_view(&self, reg: &ModuleRegistry) -> Result<ProjectView, ProjectViewError> {
        let mut data = HashMap::new();
        let mut documents = HashMap::new();
        for log_entry in &self.log {
            match log_entry {
                ProjectLogEntry::Changes {
                    session: _,
                    changes,
                } => {
                    for change in changes {
                        match change {
                            Change::CreateDocument { id } => {
                                documents.insert(*id, Document::default());
                            }
                            Change::DeleteDocument(document_id) => {
                                documents.remove_entry(document_id);
                            }
                            Change::CreateData { id, module, owner } => {
                                data.insert(
                                    *id,
                                    ErasedData {
                                        module: *module,
                                        data: (reg
                                            .0
                                            .get(module)
                                            .ok_or(ProjectViewError::UnknownModule(*module))?
                                            .init_data)(
                                        ),
                                    },
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
                            Change::MoveData {
                                id: _,
                                new_owner: _,
                            } => todo!(),
                            Change::Transaction { id, args } => {
                                let apply_transaction = reg
                                    .0
                                    .get(&args.module)
                                    .ok_or(ProjectViewError::UnknownModule(args.module))?
                                    .apply_transaction;
                                let data =
                                    data.get_mut(id).ok_or(ProjectViewError::InvalidProject)?;
                                apply_transaction(&mut data.data, &args.data);
                            }
                            Change::UserTransaction { id: _, args: _ } => todo!(),
                        }
                    }
                }
                ProjectLogEntry::Undo { session: _ } => todo!("undo/redo is not supported yet"),
                ProjectLogEntry::Redo { session: _ } => todo!("undo/redo is not supported yet"),
                ProjectLogEntry::NewSession {
                    user: _,
                    session: _,
                } => todo!(),
            };
        }

        Ok(ProjectView {
            user: User::local(),
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
    pub fn apply_changes(&mut self, cb: ChangeBuilder) {
        self.log.push(ProjectLogEntry::Changes {
            session: Uuid::new_v4(),
            changes: cb.changes,
        });
    }
}
