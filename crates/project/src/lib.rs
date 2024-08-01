//! Data and dependency management for Projects.
//!
//! Responsible for managing projects within `CADara`.
//! Projects are the primary organizational structure within `CADara`, encapsulating documents (and therefore parts and assemblies).
//! This module provides functionality to create, open, and save projects, as well as handle cross document links and dependencies inside a project.
//! This module additionally provides an API to specify a documents data structure through the [`Module`] trait.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
// TODO: allow too many lines (remove allow on functions)

// TODO: make InternalDataSession private
// TODO: rename traits/structs to not have Document in the name
// TODO: Transactions should be split into a normal and +unchecked version

// Public modules
pub mod data;
pub mod document;
pub mod manager;
pub mod user;

use data::{internal::InternalData, session::internal::InternalDataSession, DataSession};
use document::DocumentSession;
use module::Module;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;
use user::User;
use uuid::Uuid;

/// A trait for type-erased data models, enabling polymorphic handling of different data types.
///
/// This trait allows for the storage of any `DataModel` type while providing
/// mechanisms to recover the specific type through downcasting. It also facilitates
/// serialization of data without knowing their concrete types.
trait DataModelTrait: erased_serde::Serialize + Debug {
    /// Retrieves a mutable reference to the underlying type as a trait object.
    /// This is used for downcasting to the concrete `DataModel` type.
    fn as_any(&mut self) -> &mut dyn Any;
}
erased_serde::serialize_trait_object!(DataModelTrait);

/// A struct for managing shared, mutable access to an [`InternalData`].
///
/// This struct encapsulates an [`InternalData`] within `Rc<RefCell<...>>` to enable
/// shared ownership and mutability across different parts of the code. It is designed to work
/// with data models that implement the `Module` trait.
#[derive(Clone, Debug, Deserialize)]
struct DataModel<M: Module>(Rc<RefCell<InternalData<M>>>);

// We use this thread local storage to pass data to the deserialize function through
// automatically derived implementations of `Deserialize`. Alternatively, we could
// replace each step of the deserialization process with a custom implementation with a seed
// that contains the registry, but this would be more complex and less maintainable.
// TODO: look into alternatives to thread local storage
thread_local! {
    static MODULE_REGISTRY: RefCell<Option<*const ModuleRegistry>> = const { RefCell::new(None) };
}

/// A struct representing a type-erased `DataModel`.
///
/// This struct holds a `Uuid` identifying the document and a boxed `DataModelTrait`,
/// allowing for the storage and serialization of various data types without
/// knowing their concrete types at compile time.
#[derive(Debug, Serialize)]
struct ErasedDataModel {
    uuid: Uuid,
    model: Box<dyn DataModelTrait>,
}

/// Document in a Project
///
/// Defines the metadata and the identifiers of containing data sections.
#[derive(Debug, Serialize, Deserialize, Default)]
struct DocumentRecord {
    data: Vec<Uuid>,
}

// TODO: maybe custom serialization logic can be replaced with the typetag crate

impl<M: Module> DataModelTrait for DataModel<M> {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<M: Module> Serialize for DataModel<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.borrow().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ErasedDataModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Retrieve the registry from thread local storage
        // And use it to deserialize the model using the seed
        MODULE_REGISTRY.with(|r| {
            let registry = r.borrow();
            let registry = registry.expect("no registry found");
            let seed = ModuleSeed {
                // As long as the registry is alive, we can safely hold a reference to it.
                // The registry is only invalidated after deserialization is complete, so only
                // after this reference is dropped.
                registry: unsafe { &*registry },
            };
            seed.deserialize(deserializer)
        })
    }
}

/// A registry containing all installed modules necessary for deserialization.
#[derive(Clone, Debug, Default)]
pub struct ModuleRegistry {
    modules: HashMap<Uuid, BoxedDeserializeFunction<Box<dyn DataModelTrait>>>,
}

impl ModuleRegistry {
    pub fn register<M>(&mut self)
    where
        M: Module + for<'de> Deserialize<'de>,
    {
        self.modules.insert(M::uuid(), |d| {
            Ok(Box::new(erased_serde::deserialize::<DataModel<M>>(d)?))
        });
    }
}

struct ModuleSeed<'a> {
    pub registry: &'a ModuleRegistry,
}

pub struct ProjectSeed<'a> {
    pub registry: &'a ModuleRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ProjectSeed<'a>
where
    'a: 'de,
{
    type Value = Project;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Put the registry in thread local storage
        MODULE_REGISTRY.with(|r| {
            *r.borrow_mut() = Some(self.registry);
        });
        // Do the same as the derived implementation
        let o = Project::deserialize(deserializer);

        // Delete the registry from thread local storage
        MODULE_REGISTRY.with(|r| {
            *r.borrow_mut() = None;
        });
        o
    }
}

type BoxedDeserializeFunction<O> =
    for<'de> fn(&mut dyn erased_serde::Deserializer<'de>) -> Result<O, erased_serde::Error>;

struct BoxedDeserializerSeed<O: ?Sized>(pub BoxedDeserializeFunction<Box<O>>);

impl<'de, O: ?Sized> DeserializeSeed<'de> for BoxedDeserializerSeed<O> {
    type Value = Box<O>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.0(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(serde::de::Error::custom)
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ModuleSeed<'a>
where
    'a: 'de,
{
    type Value = ErasedDataModel;

    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum ModuleField {
            Uuid,
            Model,
            Ignore,
        }

        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = ModuleField;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(ModuleField::Uuid),
                    1 => Ok(ModuleField::Model),
                    _ => Ok(ModuleField::Ignore),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "uuid" => Ok(ModuleField::Uuid),
                    "model" => Ok(ModuleField::Model),
                    _ => Ok(ModuleField::Ignore),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"uuid" => Ok(ModuleField::Uuid),
                    b"model" => Ok(ModuleField::Model),
                    _ => Ok(ModuleField::Ignore),
                }
            }
        }

        impl<'de> Deserialize<'de> for ModuleField {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ModuleVisitor<'de> {
            marker: PhantomData<ErasedDataModel>,
            lifetime: PhantomData<&'de ()>,
            registry: &'de ModuleRegistry,
        }

        impl<'de> Visitor<'de> for ModuleVisitor<'de> {
            type Value = ErasedDataModel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ErasedDataModel")
            }

            #[inline]
            fn visit_seq<V>(self, mut _seq: V) -> Result<ErasedDataModel, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                // let uuid = seq
                //     .next_element()?
                //     .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                // let model = seq
                //     // .next_element_seed(ModuleSeed {
                //     //     registry: self.registry,
                //     // })?
                //     .next_element()?
                //     .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                // Ok(ErasedDataModel { uuid, model })
                todo!("sequential deserialization of ErasedDataModel is not supported yet")
            }

            #[inline]
            fn visit_map<V>(self, mut map: V) -> Result<ErasedDataModel, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut uuid = None;
                let mut model = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        ModuleField::Uuid => {
                            if uuid.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid = Some(map.next_value::<uuid::Uuid>()?);
                        }
                        ModuleField::Model => {
                            if model.is_some() {
                                return Err(serde::de::Error::duplicate_field("model"));
                            }
                            let uuid = uuid.ok_or_else(|| {
                                serde::de::Error::custom("uuid must precede model")
                            })?;
                            let d = self.registry.modules.get(&uuid).ok_or_else(|| {
                                serde::de::Error::custom("module not found in registry")
                            })?;

                            model = Some(map.next_value_seed(BoxedDeserializerSeed(*d))?);
                        }
                        ModuleField::Ignore => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(ErasedDataModel {
                    uuid: uuid.ok_or_else(|| serde::de::Error::missing_field("uuid"))?,
                    model: model.ok_or_else(|| serde::de::Error::missing_field("model"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["uuid", "model"];
        deserializer.deserialize_struct(
            "ErasedDataModel",
            FIELDS,
            ModuleVisitor {
                marker: PhantomData::<ErasedDataModel>,
                lifetime: PhantomData,
                registry: self.registry,
            },
        )
    }
}

/// Represents the internal data of a `CADara` project.
///
/// This struct is used to manage the internal state of a project, including its documents (including their data),
/// name, tags, and disk path. It is not intended for direct use by consumers of the API;
/// instead, use the [`Project`] struct for public interactions.
///
/// [`Project`]: crate::Project
#[derive(Serialize, Deserialize, Debug)]
struct InternalProject {
    /// A map linking data UUIDs to their corresponding type-erased data model.
    data: HashMap<Uuid, ErasedDataModel>,
    /// A map of all documents found in this project
    documents: HashMap<Uuid, DocumentRecord>,
    /// The name of the project.
    name: String,
    /// A list of tags associated with the project for categorization or searchability.
    tags: Vec<String>,
    /// The file system path to the project's saved location, if it has been persisted to disk.
    // TODO: implement this
    #[serde(skip)]
    _path: Option<PathBuf>,
}

/// Represents a project within the `CADara` application.
///
/// Interact with this Project through a [`ProjectSession`] by calling [`Project::create_session`].
///
/// A `Project` serves as the primary container for documents, which can represent parts,
/// assemblies, or other data units. Each document is uniquely identified by a `Uuid`.
///
/// Projects can be saved to and loaded from disk, but it is recommended to manage projects
/// through a [`ProjectManager`] to ensure data integrity, especially in multi-user scenarios.
///
/// [`ProjectManager`]: crate::manager::ProjectManager
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Project {
    /// Encapsulates the internal representation of the project, including documents and metadata.
    project: Rc<RefCell<InternalProject>>,
}

impl Project {
    //  TODO: document
    #[must_use]
    pub fn create_session(&self) -> ProjectSession {
        ProjectSession {
            project: self.project.clone(),
            user: User::local(),
        }
    }

    /// Creates a new project with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the project.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            project: Rc::new(RefCell::new(InternalProject {
                data: HashMap::new(),
                documents: HashMap::new(),
                name,
                tags: vec![],
                _path: None,
            })),
        }
    }

    /// Creates a new project given the name, user and path.
    /// TODO: replace this with a proper, maybe hide except for project manager
    #[must_use]
    pub fn new_with_path(name: String, _user: User, path: PathBuf) -> Self {
        Self {
            project: Rc::new(RefCell::new(InternalProject {
                data: HashMap::new(),
                documents: HashMap::new(),
                name,
                tags: vec![],
                _path: Some(path),
            })),
        }
    }
}

/// TODO: document
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectSession {
    /// Encapsulates the internal representation of the project, including documents and metadata.
    project: Rc<RefCell<InternalProject>>,
    /// The user currently interacting with the project.
    user: User,
}

impl ProjectSession {
    /// Opens a document
    ///
    /// # Arguments
    ///
    /// * `document_uuid` - The unique identifier of the document to open.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `DocumentSession` if the document could be opened, or `None` otherwise.
    #[must_use]
    pub fn open_document(&self, document_uuid: Uuid) -> Option<DocumentSession> {
        let project = self.project.borrow_mut();
        let _ = project.documents.get(&document_uuid)?;
        Some(DocumentSession {
            document: document_uuid,
            project: self.project.clone(),
            user: self.user,
        })
    }

    /// Creates a new empty document within the project.
    ///
    /// # Returns
    ///
    /// The unique identifier [`Uuid`] of the newly created document.
    #[must_use]
    pub fn create_document(&self) -> Uuid {
        let new_doc_uuid = Uuid::new_v4();

        let mut project = self.project.borrow_mut();
        project
            .documents
            .insert(new_doc_uuid, DocumentRecord { data: Vec::new() });
        new_doc_uuid
    }

    /// Opens a data section
    ///
    /// Given a identifier of a data sections, that is in a document inside this project,
    /// the data section can be directly be accessed, resiliant to moving of data between documents.
    ///
    /// # Arguments
    ///
    /// * `data_uuid` - The unique identifier of the data section to open.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `DataSession` if found, or `None` otherwise.
    #[must_use]
    pub fn open_data<M: Module>(&self, data_uuid: Uuid) -> Option<DataSession<M>> {
        // TODO: Option -> Result
        let project = &self.project;

        // first, we get the document model from the project (if it exists)
        let mut mut_project = project.borrow_mut();
        let data_model = mut_project
            .data
            .get_mut(&data_uuid)?
            .model
            .as_mut()
            .as_any();
        let data_model: &mut DataModel<M> = data_model.downcast_mut::<DataModel<M>>()?;

        // Create a new session for the document
        let session = InternalDataSession::new(data_model, project, data_uuid, self.user);
        Some(DataSession {
            session,
            data_model_ref: Rc::downgrade(&data_model.0),
        })
    }
}
