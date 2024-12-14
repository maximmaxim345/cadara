use crate::{
    data::{DataId, DataView},
    Change, ChangeBuilder, PendingChange, ProjectView,
};
use module::Module;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier of a `document` in a [`Project`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[expect(clippy::module_name_repetitions)]
pub struct DocumentId(Uuid);

impl DocumentId {
    /// Create a new random identifier.
    #[must_use]
    pub(crate) fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Document in a [`Project`]
///
/// Defines the metadata and the identifiers of containing data sections.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Document {
    pub data: Vec<DataId>,
}

/// A read only view to `document` in a [`ProjectView`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct DocumentView<'a> {
    pub id: DocumentId,
    pub project: &'a ProjectView,
    pub document: &'a Document,
}

impl DocumentView<'_> {
    /// Opens a read only [`DataView`] to data contained in this document.
    ///
    /// # Arguments
    /// * `data_id` - The unique identifier of the document to open
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] expected to describe the data
    ///
    /// # Returns
    /// An `Option` containing a [`DataView`] if the document was found in this document and is of type `M`, or `None` otherwise.
    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<DataView<M>> {
        if self.project.documents[&self.id]
            .data
            .iter()
            .any(|u| *u == data_id)
        {
            self.project.open_data(data_id)
        } else {
            None
        }
    }

    /// Opens read only [`DataView`]s to all data with the type `M`.
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] to filter by
    ///
    /// # Returns
    /// An iterator yielding [`DataView`]s of type `M` found in this document.
    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = DataView<M>> + '_ {
        self.project.documents[&self.id]
            .data
            .iter()
            .filter_map(|&id| self.open_data_by_id(id))
    }

    /// Plans the creation of a new empty data section with type `M`
    ///
    /// The new data section will be contained in this document
    ///
    /// This will not modify the [`Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the data recorded to `cb`.
    #[must_use]
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataId {
        let id = DataId::new_v4();
        cb.changes.push(PendingChange::Change(Change::CreateData {
            module: crate::ModuleId::from_module::<M>(),
            id,
            owner: Some(self.id),
        }));
        id
    }
}
