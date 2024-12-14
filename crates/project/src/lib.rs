//! Data and dependency management for Projects.
//!
//! [`Project`]s are the primary data structure within `CADara`. Encapsulating documents and data sections,
//! storing everything (except very short lived data in the `viewport`).
//!
//! This module provides functionality to create, open, modify, and save projects.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

// TODO: complete refactoring of project
// - adjust module to disallow errors
// - allow for errors in project, in create view and deserialization
// - add metadata, not only do documents, but also to projects and data
// - support undo/redo
// - reenable tests
// - update rest of codebase
// - Design Task (branch) > Changes (checkpoint) > Actions (change -> action, changes -> actions)
// - Revisions use CheckPoint, but also make a immutable ProjectArchive w.o redundant data
// - ChangeBuilder -> ProjectChangeSet
// - make registry functions type safe
// - reduce registry function count by splitting data

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
use module_data::{
    ErasedData, ErasedDataTransactionArgs, ErasedSessionData, ErasedSessionDataTransactionArgs,
    ErasedSharedDataTransactionArgs, ErasedUserDataTransactionArgs, ModuleId, ModuleRegistry,
    MODULE_REGISTRY,
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use user::SessionId;

// Public reexports
pub use data::DataId;
pub use data::DataView;
pub use document::DocumentId;
pub use document::DocumentView;
pub use project::ProjectView;
pub use user::UserId;

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

/// A single change to be applied to the [`Project`].
#[derive(Clone, Serialize, Deserialize, Debug)]
enum PendingChange {
    Change(Change),
    SessionTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::SessionData`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedSessionDataTransactionArgs,
    },
    SharedTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::SharedData`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedSharedDataTransactionArgs,
    },
}

/// A single change persistently applied to the [`Project`].
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
        /// Stores the [`TransactionArgs`] for [`module::Module::PersistentData`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
        args: ErasedDataTransactionArgs,
    },
    UserTransaction {
        id: DataId,
        /// Stores the [`TransactionArgs`] for [`module::Module::PersistentUserData`].
        ///
        /// The [`Module`] is equal to in the last [`Self::CreateData`] with the same [`DataId`].
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
    #[expect(clippy::too_many_lines)]
    pub fn create_view(&self, reg: &ModuleRegistry) -> Result<ProjectView, ProjectViewError> {
        let mut data = HashMap::new();
        let mut documents = HashMap::new();
        let mut sessions = HashMap::new();

        for log_entry in &self.log {
            match log_entry {
                ProjectLogEntry::Changes { session, changes } => {
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
                                let apply = reg
                                    .0
                                    .get(&args.module)
                                    .ok_or(ProjectViewError::UnknownModule(args.module))?
                                    .apply_data_transaction;
                                let data =
                                    data.get_mut(id).ok_or(ProjectViewError::InvalidProject)?;
                                apply(&mut data.data, &args.data);
                            }
                            Change::UserTransaction { id, args } => {
                                let user = *sessions
                                    .get(session)
                                    .ok_or(ProjectViewError::InvalidProject)?;

                                if self.user == user {
                                    let apply = reg
                                        .0
                                        .get(&args.module)
                                        .ok_or(ProjectViewError::UnknownModule(args.module))?
                                        .apply_user_data_transaction;
                                    let data =
                                        data.get_mut(id).ok_or(ProjectViewError::InvalidProject)?;
                                    apply(&mut data.data, &args.data);
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
            let d = data.get_mut(id).ok_or(ProjectViewError::InvalidProject)?;
            (reg.0
                .get(&session_data.module)
                .ok_or(ProjectViewError::UnknownModule(session_data.module))?
                .replace_session_data)(&mut d.data, &session_data.data);
        }

        for (id, shared_data) in &self.shared_data {
            let d = data.get_mut(id).ok_or(ProjectViewError::InvalidProject)?;
            (reg.0
                .get(&shared_data.module)
                .ok_or(ProjectViewError::UnknownModule(shared_data.module))?
                .replace_shared_data)(&mut d.data, &shared_data.data);
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
    #[expect(clippy::missing_panics_doc)]
    pub fn apply_changes(&mut self, cb: ChangeBuilder, reg: &ModuleRegistry) {
        let session = *self.session.get_or_insert_with(|| {
            let new_session = SessionId::new();
            self.log.push(ProjectLogEntry::NewSession {
                user: self.user,
                branch: self.branch,
                new_session,
            });
            new_session
        });

        let changes = cb
            .changes
            .into_iter()
            .filter_map(|change| match change {
                PendingChange::Change(change) => Some(change),
                PendingChange::SessionTransaction { id, args } => {
                    let apply = reg
                        .0
                        .get(&args.module)
                        .expect("module unknown, project is now corrupted.")
                        .apply_session_data_transaction;
                    let data = self
                        .session_data
                        .get_mut(&id)
                        .expect("project was corrupted");
                    apply(&mut data.data, &args.data);
                    None
                }
                PendingChange::SharedTransaction { id, args } => {
                    let apply = reg
                        .0
                        .get(&args.module)
                        .expect("module unknown, project is now corrupted.")
                        .apply_shared_data_transaction;
                    let data = self
                        .shared_data
                        .get_mut(&id)
                        .expect("project was corrupted");
                    apply(&mut data.data, &args.data);
                    None
                }
            })
            .collect();

        self.log.push(ProjectLogEntry::Changes { session, changes });
    }
}
