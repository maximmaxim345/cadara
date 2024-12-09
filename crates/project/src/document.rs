//! Document
//!
//! Each document is a collection of data sections, which is displayed to the user as a single item.

use crate::{
    data::{internal::InternalData, DataUuid, DataView},
    user::User,
    DataModel, ErasedDataModel, ProjectView,
};
use module::Module;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DocumentView {
    /// Identifier of this document
    pub(crate) document: DocumentUuid,
    /// Encapsulates the internal representation of the project, including documents and metadata.
    pub(crate) project: Arc<Mutex<InternalProject>>,
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

impl DocumentView {
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
        if self.project.lock().unwrap().documents[&self.document]
            .data
            .iter()
            .any(|u| *u == data_uuid)
        {
            self.project().open_data(data_uuid)
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
        let a = {
            let p = self.project.lock().unwrap();
            p.documents[&self.document].data.clone()
        };
        a.iter()
            .filter_map(|&uuid| self.open_data_by_uuid(uuid))
            .collect()
    }

    /// Returns a [`ProjectSession`] for this document's project
    #[must_use]
    pub fn project(&self) -> ProjectView {
        ProjectView {
            project: self.project.clone(),
            user: self.user,
        }
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
    pub fn create_data<M: Module>(&self) -> DataUuid {
        let new_data_uuid = DataUuid::new_v4();

        let mut project = self.project.lock().unwrap();
        let data = InternalData::<M> {
            persistent_data: M::PersistentData::default(),
            user_data: M::PersistentUserData::default(),
            sessions: vec![],
            module_uuid: M::uuid(),
            shared_data: None,
            transaction_history: std::collections::VecDeque::new(),
            session_to_user: HashMap::new(),
        };
        let data_model: DataModel<M> = DataModel(Arc::new(Mutex::new(data)));
        project.data.insert(
            new_data_uuid,
            ErasedDataModel {
                model: Box::new(data_model),
                uuid: M::uuid(),
            },
        );
        project
            .documents
            .get_mut(&self.document)
            .unwrap()
            .data
            .push(new_data_uuid);
        new_data_uuid
    }
}
