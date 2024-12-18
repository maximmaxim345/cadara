use std::{borrow::Cow, fmt, marker::PhantomData, ops::Deref};

use crate::{
    data::{DataId, DataView, PlannedData},
    Change, ChangeBuilder, PendingChange, ProjectSource, ProjectView,
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
    /// Unique identifier to associalte a project with its views and [`ChangeBuilder`]s
    pub(crate) uuid: uuid::Uuid,
}

impl ProjectSource for DocumentView<'_> {
    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
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
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    #[must_use]
    pub fn create_data<'a, 'b, M: Module>(
        &'a self,
        cb: &'b mut ChangeBuilder,
    ) -> PlannedData<'a, 'b, M> {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
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
            cb,
        }
    }

    /// Plans the deletion of this document and all its contained data
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        assert!(
            cb.is_same_source_as(self),
            "ChangeBuilder must stem from the same project"
        );
        cb.changes
            .push(PendingChange::Change(Change::DeleteDocument(self.id)));
    }
}

/// Pending version of [`DocumentView`] that does not yet exist in the [`ProjectView`].
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct PlannedDocument<'a, 'b> {
    pub id: DocumentId,
    pub project: &'a ProjectView,
    pub(crate) cb: &'b mut ChangeBuilder,
}

impl From<PlannedDocument<'_, '_>> for DocumentId {
    fn from(dv: PlannedDocument<'_, '_>) -> Self {
        dv.id
    }
}

impl Deref for PlannedDocument<'_, '_> {
    type Target = DocumentId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl PlannedDocument<'_, '_> {
    /// Plans the creation of a new empty data section with type `M`
    ///
    /// The new data section will be contained in this document
    ///
    /// This will not modify the [`crate::Project`], just record this change to the [`ChangeBuilder`]
    /// used to create this [`PlannedDocument`].
    ///
    /// # Returns
    /// The unique identifier of the data recorded to the [`ChangeBuilder`].
    #[must_use]
    pub fn create_data<M: Module>(&mut self) -> PlannedData<'_, '_, M> {
        let id = DataId::new_v4();
        self.cb
            .changes
            .push(PendingChange::Change(Change::CreateData {
                module: crate::ModuleId::from_module::<M>(),
                id,
                owner: Some(self.id),
            }));
        PlannedData {
            id,
            project: self.project,
            phantomdata: PhantomData::<M>,
            cb: self.cb,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PathCreationError {
    #[error("'{0} is not a valid Path")]
    InvalidPath(String),
}

pub struct AncestorsUnescapedIter<'a> {
    path: &'a str,
    end_index: Option<usize>,
}

impl<'a> Iterator for AncestorsUnescapedIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(a) = self.end_index {
            self.path = &self.path[a..];
        }
        let a = self.path.char_indices().find_map(|(i, char)| match char {
            '/' => {
                let mut escaped = false;
                for prev_c in self.path[..i].chars().rev() {
                    if prev_c == '\\' {
                        escaped = !escaped;
                    } else {
                        break;
                    }
                }

                if escaped {
                    None
                } else {
                    Some(i)
                }
            }
            _ => None,
        });
        if let Some(a) = a {
            let b = &self.path[..a];
            self.end_index = Some(a + 1);
            Some(b)
        } else {
            None
        }
    }
}

pub struct AncestorsIter<'a>(AncestorsUnescapedIter<'a>);

impl<'a> Iterator for AncestorsIter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Path::unescape(self.0.next()?))
    }
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
            if char == '\\' {
                escaped = !escaped;
                last_char_was_slash = false;
            } else if escaped {
                if char != '/' {
                    // Lone '\' is not allowed
                    return Err(PathCreationError::InvalidPath(path));
                }
                escaped = false;
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

    /// Returns the unescaped name of the document/folder of this [`Path`].
    ///
    /// Get the unescaped Name of this [`Path`], corresponding to the last component separated
    /// with a `/`. The returned `&str` still has escaped characters, so `\` will be `\\` and
    /// `/` will be `\/`, so use [`Self::get_name`] in case this is a problem.
    #[must_use]
    pub fn get_name_escaped(&self) -> &str {
        let path = &self.0;
        let beginning = path
            .char_indices()
            .rev()
            .find_map(|(i, c)| match c {
                '/' => {
                    let mut escaped = false;
                    for prev_c in path[..i].chars().rev() {
                        if prev_c == '\\' {
                            escaped = !escaped;
                        } else {
                            break;
                        }
                    }

                    if escaped {
                        None
                    } else {
                        Some(i + 1)
                    }
                }
                _ => None,
            })
            .unwrap_or(0); // Default to 0 if no unescaped '/' is found
        &path[beginning..]
    }

    /// Iterate through all parent folders of this document. Unescaped Version.
    ///
    /// Iterate through unescaped folder names of this [`Path`], starting from the root.
    /// The returned `&str` still has escaped characters, so `\` will be `\\` and
    /// `/` will be `\/`.
    ///
    /// Use [`Self::ancestors`] or convert the `&str`s with [`Self::unescape`] to
    /// convert it to the correct folder name.
    #[must_use]
    pub fn ancestors_unescaped(&self) -> AncestorsUnescapedIter {
        AncestorsUnescapedIter {
            path: &self.0[1..],
            end_index: None,
        }
    }

    /// Iterate through all parent folders of this document.
    ///
    /// Iterate through folder names of this [`Path`], starting from the root.
    ///
    /// This function might allocate. If this is not desired, use [`Self::ancestors_unescaped`] instead.
    #[must_use]
    pub fn ancestors(&self) -> AncestorsIter {
        AncestorsIter(AncestorsUnescapedIter {
            path: &self.0[1..],
            end_index: None,
        })
    }

    /// Returns the name of the document/folder of this [`Path`].
    ///
    /// Get the Name of this [`Path`], corresponding to the last component separated
    /// with a `/`.
    ///
    /// This function might allocate. If this is not desired, use [`Self::get_name_escaped`] instead.
    #[must_use]
    pub fn get_name(&self) -> Cow<'_, str> {
        Self::unescape(self.get_name_escaped())
    }

    /// Unescapes a string by removing backslashes before escaped characters.
    #[must_use]
    pub fn unescape(escaped: &str) -> Cow<'_, str> {
        let escape_count = escaped.chars().filter(|&c| c == '\\').count();
        if escape_count == 0 {
            Cow::Borrowed(escaped)
        } else {
            let mut unescaped = String::with_capacity(escaped.len() - escape_count);

            let mut chars = escaped.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(next_char) = chars.next() {
                        unescaped.push(next_char);
                    }
                } else {
                    unescaped.push(c);
                }
            }

            Cow::Owned(unescaped)
        }
    }

    /// Escapes a string by adding backslashes before backslashes and slashes
    #[must_use]
    pub fn escape(string: &str) -> Cow<'_, str> {
        // TODO: proptest???
        let escape_count = string.chars().filter(|&c| c == '\\' || c == '/').count();
        if escape_count == 0 {
            Cow::Borrowed(string)
        } else {
            let mut unescaped = String::with_capacity(string.len() + escape_count);

            for c in string.chars() {
                if c == '\\' || c == '/' {
                    unescaped.push('\\');
                }
                unescaped.push(c);
            }

            Cow::Owned(unescaped)
        }
    }

    /// Increments the numeric suffix of a name, or adds one if not present.
    #[must_use]
    pub fn increment_name_suffix(&self) -> Self {
        // Split at last parenthesis pair if it exists
        if let Some((name, num)) = self.0.rsplit_once(" (").and_then(|(name, end)| {
            end.strip_suffix(')')
                .and_then(|num| num.parse::<usize>().ok())
                .map(|num| (name, num))
        }) {
            // If we found a valid number suffix, increment it
            Self(format!("{name} ({})", num + 1))
        } else {
            // If no number suffix exists, add (2)
            Self(format!("{} (2)", self.0))
        }
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
