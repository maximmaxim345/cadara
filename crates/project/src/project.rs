use crate::{
    data::{DataId, DataView},
    document::{Document, DocumentId, DocumentView, PlannedDocument},
    module_data::{ErasedData, ModuleId},
    user::UserId,
    Change, ChangeBuilder, Path, PendingChange, ProjectSource,
};
use module::Module;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// A read-only snapshot of a [`crate::Project`] at a specific point in time.
///
/// [`ProjectView`] provides a consistent and immutable view of the project's state, including all documents and data sections.
///
/// Use [`crate::Project::create_view`] create a [`ProjectView`].
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectView {
    /// The user currently interacting with the project.
    pub(crate) user: UserId,
    /// A map containing all [`crate::module_data::Data`]
    pub(crate) data: HashMap<DataId, ErasedData>,
    /// A map of all documents found in this project
    pub(crate) documents: HashMap<DocumentId, Document>,
    /// Unique identifier to associate a project with its views and [`ChangeBuilder`]s
    #[serde(skip)]
    pub(crate) uuid: uuid::Uuid,
}

impl ProjectSource for ProjectView {
    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

impl ProjectSource for std::sync::Arc<ProjectView> {
    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

impl ProjectView {
    /// Opens a read only [`DocumentView`].
    ///
    /// # Arguments
    /// * `document_id` - The unique identifier of the document to open
    ///
    /// # Returns
    /// An `Option` containing a [`DocumentView`] if the document was found, or `None` otherwise.
    #[must_use]
    pub fn open_document(&self, document_id: DocumentId) -> Option<DocumentView> {
        Some(DocumentView {
            id: document_id,
            project: self,
            document: self.documents.get(&document_id)?,
            uuid: self.uuid,
        })
    }

    /// Plans the creation of a new empty document.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the document recorded to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    #[must_use]
    pub fn create_document<'a, 'b>(
        &'a self,
        cb: &'b mut ChangeBuilder,
        path: Path,
    ) -> PlannedDocument<'a, 'b> {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        let id = DocumentId::new_v4();

        cb.changes
            .push(PendingChange::Change(Change::CreateDocument { id, path }));
        PlannedDocument {
            id,
            project: self,
            cb,
        }
    }

    /// Plans the creation of a new empty data section with type `M`.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    ///
    /// The unique identifier of the data recorded to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataId {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        let id = DataId::new_v4();
        cb.changes.push(PendingChange::Change(Change::CreateData {
            module: ModuleId::from_module::<M>(),
            id,
            owner: None,
        }));
        id
    }

    /// Opens a read only [`DataView`].
    ///
    /// # Arguments
    /// * `data_id` - The unique identifier of the document to open
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] expected to describe the data
    ///
    /// # Returns
    /// An `Option` containing a [`DataView`] if the document was found and is of type `M`, or `None` otherwise.
    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<DataView<M>> {
        // TODO: Option -> Result
        let data = &self.data.get(&data_id)?.downcast_ref::<M>()?;

        Some(DataView {
            project: self,
            id: data_id,
            persistent: &data.persistent,
            persistent_user: &data.persistent_user,
            session_data: &data.session,
            shared_data: &data.shared,
            uuid: self.uuid,
        })
    }

    /// Opens read only [`DataView`]s to all data with the type `M`.
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] to filter by
    ///
    /// # Returns
    /// An iterator yielding [`DataView`]s of type `M` found in this document.
    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = DataView<M>> + '_ {
        self.data.keys().filter_map(|id| self.open_data_by_id(*id))
    }
}
