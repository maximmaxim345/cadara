//! Document
//!
//! Each document is a collection of data sections, which is displayed to the user as a single item.

use crate::{
    data::{DataUuid, DataView},
    user::User,
    ChangeBuilder, DataModel, ErasedDataModel, ProjectLogEntry, ProjectView,
};
use module::Module;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DocumentView<'a> {
    /// Identifier of this document
    pub(crate) document: DocumentUuid,
    pub project: &'a ProjectView,
    /// The user currently interacting with the project.
    pub(crate) user: User,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct DocumentUuid {
    uuid: Uuid,
}

impl DocumentUuid {
    // TODO: make this pub(crate)
    #[must_use]
    pub const fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    #[must_use]
    pub fn new_v4() -> Self {
        Self::new(Uuid::new_v4())
    }
}

impl<'a> DocumentView<'a> {
    /// Opens a data section contained in this document by UUID
    ///
    /// # Arguments
    ///
    /// * `data_uuid` - The unique identifier of the data section to open
    ///
    /// # Returns
    ///
    /// An `Option` containing a `DataSession` if the data section exists, or `None` otherwise.
    #[must_use]
    pub fn open_data_by_uuid<M: Module>(&self, data_uuid: DataUuid) -> Option<DataView<M>> {
        if self.project.documents[&self.document]
            .data
            .iter()
            .any(|u| *u == data_uuid)
        {
            self.project.open_data(data_uuid)
        } else {
            None
        }
    }

    /// Opens all data sections of a specific type in this document
    ///
    /// # Returns
    ///
    /// A vector containing a [`DataSession`] for each data section of the type `M` found
    /// in this document.
    ///
    /// TODO: make this an iterator, or return an Newtype of Uuid
    #[must_use]
    pub fn open_data_by_type<M: Module>(&self) -> Vec<DataView<M>> {
        self.project.documents[&self.document]
            .data
            .iter()
            .filter_map(|&uuid| self.open_data_by_uuid(uuid))
            .collect()
    }

    /// Creates a new data section inside this document
    ///
    /// # Returns
    ///
    /// The project-wide unique identifier [`Uuid`] of the newly created data section.
    ///
    /// # Panics
    ///
    /// If the document was deleted after creating this session object.
    #[must_use]
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataUuid {
        let uuid = DataUuid::new_v4();
        cb.changes.push(ProjectLogEntry::CreateData {
            t: M::uuid(),
            uuid,
            owner: Some(self.document),
        });
        uuid
    }
}
