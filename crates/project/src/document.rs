use std::{fmt, marker::PhantomData, ops::Deref};

use crate::{
    data::{DataId, DataView, PlannedData},
    Change, ChangeBuilder, PendingChange, ProjectView,
};
use module::Module;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// Unique identifier of a `document` in a [`crate::Project`].
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

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocumentId({})", self.0)
    }
}

/// Document in a [`crate::Project`]
///
/// Defines the metadata and the identifiers of containing data sections.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Document {
    pub data: Vec<DataId>,
}

/// A read-only view to a `Document` within a [`ProjectView`].
///
/// [`DocumentView`] provides access to the metadata and data sections contained within a specific document
/// in a project. It allows you to inspect the document's contents and create data sections, but not to modify the project directly.
/// Modifications must be made through a [`ChangeBuilder`] and applied to the [`crate::Project`] using [`crate::Project::apply_changes`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct DocumentView<'a> {
    pub id: DocumentId,
    pub project: &'a ProjectView,
    pub(crate) document: &'a Document,
}

impl From<DocumentView<'_>> for DocumentId {
    fn from(dv: DocumentView<'_>) -> Self {
        dv.id
    }
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
        if self.document.data.iter().any(|u| *u == data_id) {
            self.project.open_data_by_id(data_id)
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
        self.document
            .data
            .iter()
            .filter_map(|&id| self.open_data_by_id(id))
    }

    /// Plans the creation of a new empty data section with type `M`
    ///
    /// The new data section will be contained in this document
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the data recorded to `cb`.
    #[must_use]
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> PlannedData<'_, M> {
        let id = DataId::new_v4();
        cb.changes.push(PendingChange::Change(Change::CreateData {
            module: crate::ModuleId::from_module::<M>(),
            id,
            owner: Some(self.id),
        }));
        PlannedData {
            id,
            project: self.project,
            phantomdata: PhantomData::<M>,
        }
    }

    /// Plans the deletion of this document and all its contained data
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        cb.changes
            .push(PendingChange::Change(Change::DeleteDocument(self.id)));
    }
}

/// Pending version of [`DocumentView`] that does not yet exist in the [`ProjectView`].
#[derive(Clone, Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct PlannedDocument<'a> {
    pub id: DocumentId,
    pub project: &'a ProjectView,
    pub(crate) phantomdata: PhantomData<()>,
}

impl From<PlannedDocument<'_>> for DocumentId {
    fn from(dv: PlannedDocument<'_>) -> Self {
        dv.id
    }
}

impl Deref for PlannedDocument<'_> {
    type Target = DocumentId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl PlannedDocument<'_> {
    /// Plans the creation of a new empty data section with type `M`
    ///
    /// The new data section will be contained in this document
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the data recorded to `cb`.
    #[must_use]
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> PlannedData<'_, M> {
        let id = DataId::new_v4();
        cb.changes.push(PendingChange::Change(Change::CreateData {
            module: crate::ModuleId::from_module::<M>(),
            id,
            owner: Some(self.id),
        }));
        PlannedData {
            id,
            project: self.project,
            phantomdata: PhantomData::<M>,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PathCreationError {
    #[error("'{0} is not a valid Path")]
    InvalidPath(String),
}

/// Path of a document/folder in a [`crate::Project`].
///
/// A [`Path`] should uniquely identify the location of a document or folder
/// (excluding the root folder) inside a [`crate::Project`] and consists of two parts:
/// location and name, with `/` used as a separator.
///
/// The location must start with `/`, with  `/` indicating the location in the folder.
///
/// A `/` can be escaped using `\/` (`\\` is `\`)
///
/// Examples for valid paths:
/// - `/part` => `part` in root folder
/// - `/assemblies and drawings/drawing` => `drawing` in `assemblies and drawings`
/// - `/parts/screws\/bolts/bolt1` => `bolt1` in `parts` / `screws/bolts`
///
/// Invalid paths:
/// - `part`
/// - `/parts/`
/// - `//part`
/// - `/`
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct Path(String);

impl TryFrom<String> for Path {
    type Error = PathCreationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Path {
    type Error = PathCreationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

impl Path {
    /// Try to create a new [`Path`] from a [`String`].
    ///
    /// # Errors
    /// If the `path` is not a valid [`Path`] identifier.
    pub fn new(path: String) -> Result<Self, PathCreationError> {
        if !path.starts_with('/') {
            return Err(PathCreationError::InvalidPath(path));
        }

        let mut escaped = false;
        let mut last_char_was_slash = false;
        for char in path.chars() {
            if escaped {
                escaped = false;
                last_char_was_slash = false;
            } else if char == '\\' {
                escaped = true;
                last_char_was_slash = false;
            } else if char == '/' {
                if last_char_was_slash {
                    // Two '/' are not allowed
                    return Err(PathCreationError::InvalidPath(path));
                }
                last_char_was_slash = true;
            } else {
                last_char_was_slash = false;
            }
        }
        if last_char_was_slash || escaped {
            // Path must not end with a '/' or unescaped '\'
            return Err(PathCreationError::InvalidPath(path));
        }
        Ok(Self(path))
    }

    /// Gets the string representation of the Path
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Increments the numeric suffix of a name, or adds one if not present.
    #[must_use]
    pub fn increment_name_suffix(&self) -> Self {
        todo!("implement")
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Self::new(path).map_err(serde::de::Error::custom)
    }
}
