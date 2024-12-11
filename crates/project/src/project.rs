use crate::data::DataId;
use crate::data::DataView;
use crate::document::Document;
use crate::document::DocumentId;
use crate::document::DocumentView;
use crate::module_data::ErasedData;
use crate::module_data::ModuleId;
use crate::user::User;
use crate::Change;
use crate::ChangeBuilder;
use module::Module;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// A read only view to all data stored in a [`Project`].
///
/// Use [`Project::create_view`] create a [`ProjectView`].
#[derive(Clone, Serialize, Deserialize, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct ProjectView {
    /// The user currently interacting with the project.
    pub user: User,
    /// A map containing all [`Data`]
    pub data: HashMap<DataId, ErasedData>,
    /// A map of all documents found in this project
    pub documents: HashMap<DocumentId, Document>,
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
        })
    }

    /// Plans the creation of a new empty document.
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the document recorded to `cb`.
    #[expect(clippy::unused_self)]
    #[must_use]
    pub fn create_document(&self, cb: &mut ChangeBuilder) -> DocumentId {
        let id = DocumentId::new_v4();

        cb.changes.push(Change::CreateDocument { id });
        id
    }

    /// Plans the creation of a new empty data section with type `M`.
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Returns
    ///
    /// The unique identifier of the data recorded to `cb`.
    #[expect(clippy::unused_self, reason = "for a consistent API")]
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataId {
        let id = DataId::new_v4();
        cb.changes.push(Change::CreateData {
            module: ModuleId::from_module::<M>(),
            id,
            owner: None,
        });
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
    pub fn open_data<M: Module>(&self, data_id: DataId) -> Option<DataView<M>> {
        // TODO: Option -> Result
        let data = &self.data.get(&data_id)?.downcast_ref::<M>()?;

        Some(DataView {
            project: self,
            id: data_id,
            persistent: &data.persistent,
            persistent_user: &data.persistent_user,
            session_data: &data.session,
            shared_data: &data.shared,
        })
    }
}
