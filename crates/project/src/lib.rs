//! Data and dependency management for Projects.
//!
//! Responsible for managing projects within `CADara`.
//! Projects are the primary organizational structure within `CADara`, encapsulating documents and data sections (i.e. parts and assemblies).
//! This module provides functionality to create, open, and save projects.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
// #![allow(clippy::module_name_repetitions)] // we don't want 3 different `Session` types
// #![allow(clippy::cognitive_complexity)]
// #![allow(clippy::missing_panics_doc)] // TODO: delete this asap

// TODO: Transactions should be split into a normal and +unchecked version

// Public modules
pub mod data;
pub mod document;
pub mod user;

use data::DataUuid;
use data::DataView;
use document::{DocumentUuid, DocumentView};
use dyn_clone::DynClone;
use module::{DataTransaction, Module};
use paste::paste;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use user::User;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
struct ModuleUuid(Uuid);

impl ModuleUuid {
    pub fn from_module<M: Module>() -> Self {
        Self(M::uuid())
    }
}

/// Complete state of a data of a module, accessable through a [`DataView`].
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct Data<M: Module> {
    pub persistent: M::PersistentData,
    pub persistent_user: M::PersistentUserData,
    pub session: M::SessionData,
    pub shared: M::SharedData,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SharedData<M: Module>(M::SharedData);

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SessionData<M: Module>(M::SessionData);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TransactionData<M: Module>(<M::PersistentData as DataTransaction>::Args);

/// Generates type-erased data container implementations for a given, on [`Module`] generic, data type
///
/// This macro implements:
/// - Serialization and deserialization
/// - A dynamic wrapper type that provides type-safe downcasting
/// - Clone for dynamic types
///
/// # Arguments
///
/// * `$d` - The base data type to generate implementations for
/// * `$reg_entry` - Name of the [`BoxedDeserializeFunction`] in [`ModuleRegEntry`]
///
/// # Generated Types
///
/// For a data type `T`, this macro generates:
/// - `TTrait` - A trait implementing common behavior
/// - `DynT` - A type-erased wrapper with serialization support
/// - `TDeserializeSeed` - Custom deserializer for the type-erased data
macro_rules! define_type_erased_data {
    ($d:ty, $reg_entry:ident) => {
        paste! {
            #[doc = "A trait shared by all [`" $d "`] types for all [`Module`]"]
            #[allow(dead_code)]
            trait [<$d Trait>]: erased_serde::Serialize + Debug + Send + Any + DynClone {
                /// Provides read-only access to the underlying data type.
                fn as_any(&self) -> &dyn Any;
                /// Provides mutable access to the underlying data type.
                fn as_mut_any(&mut self) -> &mut dyn Any;
            }

            dyn_clone::clone_trait_object!([<$d Trait>]);
            erased_serde::serialize_trait_object!([<$d Trait>]);

            impl<M: Module> [<$d Trait>] for $d<M> {
                fn as_mut_any(&mut self) -> &mut dyn Any {
                    self
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }
            }

            #[doc = "Serializable, Deserializable and Cloneable wrapper around all generic [`" $d "`] types."]
            #[derive(Debug, Serialize, Clone)]
            struct [<Dyn $d>] {
                // uuid of the module, over that the struct contained in `data` is generic
                module: ModuleUuid,
                #[doc = "Type erased [`" $d "`]"]
                data: Box<dyn [<$d Trait>]>,
            }

            #[allow(dead_code)]
            impl [<Dyn $d>] {
                pub fn downcast_ref<M: Module>(&self) -> Option<&$d<M>> {
                    self.data.as_any().downcast_ref()
                }

                pub fn downcast_mut<M: Module>(&mut self) -> Option<&mut $d<M>> {
                    self.data.as_mut_any().downcast_mut()
                }

            }

            impl<M: Module> From<$d<M>> for [<Dyn $d>] {
                fn from(d: $d<M>) -> Self {
                    Self {
                        module: ModuleUuid::from_module::<M>(),
                        data: Box::new(d),
                    }
                }
            }

            impl<'de> Deserialize<'de> for [<Dyn $d>] {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    // Retrieve the registry from thread local storage
                    // And use it to deserialize the model using the seed
                    MODULE_REGISTRY.with(|r| {
                        let registry = r.borrow();
                        let registry = registry.expect("no registry found");
                        let seed = [<$d DeserializeSeed>] {
                            // SAFETY: As long as the registry is alive, we can safely hold a reference to it.
                            // The registry is only invalidated after deserialization is complete, so only
                            // after this reference is dropped.
                            registry: unsafe { &*registry },
                        };
                        seed.deserialize(deserializer)
                    })
                }
            }

            struct [<$d DeserializeSeed>]<'a> {
                pub registry: &'a ModuleRegistry,
            }

            // We manually implement deserialization logic to support runtime polymorphism
            // The `typetag` could do this for us, but it unfortunately does not support WebAssembly
            impl<'a, 'de> DeserializeSeed<'de> for [<$d DeserializeSeed>]<'a>
            where
                'a: 'de,
            {
                type Value = [<Dyn $d>];

                #[expect(clippy::too_many_lines)]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    enum ModuleField {
                        Module,
                        Data,
                        Ignore,
                    }

                    struct FieldVisitor;

                    impl Visitor<'_> for FieldVisitor {
                        type Value = ModuleField;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("field identifier")
                        }

                        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                0 => Ok(ModuleField::Module),
                                1 => Ok(ModuleField::Data),
                                _ => Ok(ModuleField::Ignore),
                            }
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                "module" => Ok(ModuleField::Module),
                                "data" => Ok(ModuleField::Data),
                                _ => Ok(ModuleField::Ignore),
                            }
                        }

                        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                b"module" => Ok(ModuleField::Module),
                                b"data" => Ok(ModuleField::Data),
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
                        marker: PhantomData<[<Dyn $d>]>,
                        lifetime: PhantomData<&'de ()>,
                        registry: &'de ModuleRegistry,
                    }

                    impl<'de> Visitor<'de> for ModuleVisitor<'de> {
                        type Value = [<Dyn $d>];

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(concat!("struct ", stringify!([<Dyn $d>])))
                        }

                        #[inline]
                        fn visit_seq<V>(self, mut _seq: V) -> Result<[<Dyn $d>], V::Error>
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
                            todo!("sequential deserialization is not supported yet")
                        }

                        #[inline]
                        fn visit_map<V>(self, mut map: V) -> Result<[<Dyn $d>], V::Error>
                        where
                            V: serde::de::MapAccess<'de>,
                        {
                            let mut module = None;
                            let mut data = None;
                            while let Some(key) = map.next_key()? {
                                match key {
                                    ModuleField::Module => {
                                        if module.is_some() {
                                            return Err(serde::de::Error::duplicate_field("module"));
                                        }
                                        module = Some(map.next_value::<ModuleUuid>()?);
                                    }
                                    ModuleField::Data => {
                                        if data.is_some() {
                                            return Err(serde::de::Error::duplicate_field("data"));
                                        }
                                        let module = module.ok_or_else(|| {
                                            serde::de::Error::custom("module must precede data")
                                        })?;
                                        let d = self.registry.0.get(&module).ok_or_else(|| {
                                            serde::de::Error::custom("module not found in registry")
                                        })?.$reg_entry;

                                        data = Some(map.next_value_seed(BoxedDeserializerSeed(d))?);
                                    }
                                    ModuleField::Ignore => {
                                        let _: serde::de::IgnoredAny = map.next_value()?;
                                    }
                                }
                            }
                            Ok([<Dyn $d>] {
                                module: module.ok_or_else(|| serde::de::Error::missing_field("module"))?,
                                data: data.ok_or_else(|| serde::de::Error::missing_field("data"))?,
                            })
                        }
                    }

                    const FIELDS: &[&str] = &["module", "data"];
                    deserializer.deserialize_struct(
                        stringify!([<Dyn $d>]),
                        FIELDS,
                        ModuleVisitor {
                            marker: PhantomData::<[<Dyn $d>]>,
                            lifetime: PhantomData,
                            registry: self.registry,
                        },
                    )
                }
            }
        }
    };
}

define_type_erased_data!(Data, deserialize_data);
define_type_erased_data!(SessionData, deserialize_session);
define_type_erased_data!(SharedData, deserialize_shared);
define_type_erased_data!(TransactionData, deserialize_transaction);

// We use this thread local storage to pass data to the deserialize function through
// automatically derived implementations of `Deserialize`. Alternatively, we could
// replace each step of the deserialization process with a custom implementation with a seed
// that contains the registry, but this would be more complex and less maintainable.
// TODO: look into alternatives to thread local storage
thread_local! {
    static MODULE_REGISTRY: RefCell<Option<*const ModuleRegistry>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum TransactionTarget {
    PersistentData(DataUuid),
    PersistendUserData(DataUuid, User),
}

/// Document in a Project
///
/// Defines the metadata and the identifiers of containing data sections.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct DocumentRecord {
    data: Vec<DataUuid>,
}

#[derive(Clone, Debug)]
struct ModuleRegEntry {
    deserialize_data: BoxedDeserializeFunction<Box<dyn DataTrait>>,
    deserialize_transaction: BoxedDeserializeFunction<Box<dyn TransactionDataTrait>>,
    deserialize_shared: BoxedDeserializeFunction<Box<dyn SharedDataTrait>>,
    deserialize_session: BoxedDeserializeFunction<Box<dyn SessionDataTrait>>,
    init_data: fn() -> Box<dyn DataTrait>,
    apply_transaction: fn(&mut Box<dyn DataTrait>, &Box<dyn TransactionDataTrait>),
}

/// A registry containing all installed modules necessary for deserialization.
#[derive(Clone, Debug, Default)]
pub struct ModuleRegistry(HashMap<ModuleUuid, ModuleRegEntry>);

impl ModuleRegistry {
    pub fn register<M>(&mut self)
    where
        M: Module + for<'de> Deserialize<'de>,
    {
        self.0.insert(
            ModuleUuid::from_module::<M>(),
            ModuleRegEntry {
                deserialize_data: |d| Ok(Box::new(erased_serde::deserialize::<Data<M>>(d)?)),
                deserialize_transaction: |d| {
                    Ok(Box::new(erased_serde::deserialize::<TransactionData<M>>(
                        d,
                    )?))
                },
                deserialize_shared: |d| {
                    Ok(Box::new(erased_serde::deserialize::<SharedData<M>>(d)?))
                },
                deserialize_session: |d| {
                    Ok(Box::new(erased_serde::deserialize::<SessionData<M>>(d)?))
                },
                init_data: || Box::new(Data::<M>::default()),
                apply_transaction: |m, t| {
                    let m = m.as_mut().as_mut_any().downcast_mut::<Data<M>>().unwrap();
                    // TODO: persistent user is not implemented
                    let t = t
                        .as_ref()
                        .as_any()
                        .downcast_ref::<TransactionData<M>>()
                        .unwrap();
                    module::DataTransaction::apply(&mut m.persistent, t.0.clone());
                },
            },
        );
    }
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

#[derive(Clone, Serialize, Deserialize, Debug)]
enum Change {
    CreateDocument {
        uuid: DocumentUuid,
        name: String,
    },
    DeleteDocument(DocumentUuid),
    RenameDocument {
        uuid: DocumentUuid,
        new_name: String,
    },
    CreateData {
        module: ModuleUuid,
        uuid: DataUuid,
        owner: Option<DocumentUuid>,
    },
    DeleteData {
        uuid: DataUuid,
    },
    MoveData {
        uuid: DataUuid,
        new_owner: Option<DocumentUuid>,
    },
    Transaction {
        target: TransactionTarget,
        data: DynTransactionData,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum ProjectLogEntry {
    Change { session: Uuid, entries: Vec<Change> },
    Undo { session: Uuid },
    Redo { session: Uuid },
    NewSession { user: User, session: Uuid },
}

#[derive(Clone, Debug, Default)]
pub struct ChangeBuilder {
    changes: Vec<Change>,
}

impl ChangeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, mut other: Self) {
        self.changes.append(&mut other.changes);
    }
}

/// Represents a project within the `CADara` application.
///
/// Interact with this Project through a [`ProjectSession`] by calling [`Project::create_session`].
///
/// A [`Project`] serves as the primary container for documents, which can represent parts,
/// assemblies, or other data units. Each document is uniquely identified by a [`Uuid`].
///
/// Projects can be saved to and loaded from disk, but it is recommended to manage projects
/// through a [`ProjectManager`] to ensure data integrity, especially in multi-user scenarios.
///
/// [`ProjectManager`]: crate::manager::ProjectManager
// TODO: remove `Project` and rename `InternalProject` to `Project`
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Project {
    /// Chronological list of all applied [`ProjectTransaction`]s.
    log: Vec<ProjectLogEntry>,
    shared_data: HashMap<DataUuid, DynSharedData>,
    session_data: HashMap<DataUuid, DynSessionData>,
}

impl Project {
    //  TODO: document
    #[must_use]
    pub fn create_view(&self, reg: &ModuleRegistry) -> ProjectView {
        let mut data = HashMap::new();
        let mut documents = HashMap::new();
        for e in &self.log {
            match e {
                ProjectLogEntry::Change { session, entries } => {
                    for e in entries {
                        match e {
                            Change::CreateDocument { uuid, name } => {
                                documents.insert(*uuid, DocumentRecord::default());
                            }
                            Change::DeleteDocument(document_uuid) => {
                                documents.remove_entry(document_uuid);
                            }
                            Change::RenameDocument { uuid, new_name } => {}
                            Change::CreateData {
                                module: t,
                                uuid,
                                owner,
                            } => {
                                data.insert(
                                    *uuid,
                                    DynData {
                                        module: *t,
                                        data: (reg.0.get(t).unwrap().init_data)(),
                                    },
                                );
                                if let Some(owner) = owner {
                                    documents
                                        .entry(*owner)
                                        .or_insert(Default::default())
                                        .data
                                        .push(*uuid);
                                }
                            }
                            Change::DeleteData { uuid } => {
                                data.remove(uuid);
                            }
                            Change::MoveData { uuid, new_owner } => todo!(),
                            Change::Transaction { target, data: d } => {
                                let apply = reg.0.get(&d.module).unwrap().apply_transaction;
                                match target {
                                    TransactionTarget::PersistentData(data_uuid) => {
                                        let d2 = data.get_mut(data_uuid).unwrap();
                                        // TODO: assert if correct
                                        apply(&mut d2.data, &d.data);
                                    }
                                    TransactionTarget::PersistendUserData(data_uuid, user) => {
                                        todo!("add support for this, currently the trait does not support this")
                                    }
                                }
                            }
                        }
                    }
                }
                ProjectLogEntry::Undo { session } => todo!(),
                ProjectLogEntry::Redo { session } => todo!(),
                ProjectLogEntry::NewSession { user, session } => todo!(),
            };
        }

        ProjectView {
            user: User::local(),
            data,
            documents,
        }
    }

    /// Creates a new project with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the project.
    #[must_use]
    pub fn new(_name: String) -> Self {
        Self::default()
    }

    /// Creates a new project given the name, user and path.
    /// TODO: replace this with a proper, maybe hide except for project manager
    #[must_use]
    pub fn new_with_path(_name: String, _user: User, _path: PathBuf) -> Self {
        todo!("remove or implement this")
    }

    pub fn apply_changes(&mut self, cb: ChangeBuilder) {
        self.log.push(ProjectLogEntry::Change {
            session: Uuid::new_v4(),
            entries: cb.changes,
        });
    }
}

/// TODO: document
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectView {
    /// The user currently interacting with the project.
    user: User,
    /// TODO: document
    data: HashMap<DataUuid, DynData>,
    /// A map of all documents found in this project
    documents: HashMap<DocumentUuid, DocumentRecord>,
}

impl ProjectView {
    /// Opens a document
    ///
    /// # Arguments
    ///
    /// * `document_uuid` - The unique identifier of the document to open.
    ///
    /// # Returns
    ///
    /// An `Option` containing a [`DocumentSession`] if the document could be opened, or `None` otherwise.
    #[must_use]
    pub const fn open_document(&self, document_uuid: DocumentUuid) -> Option<DocumentView> {
        Some(DocumentView {
            document: document_uuid,
            project: self,
        })
    }

    /// Creates a new empty document within the project.
    ///
    /// # Returns
    ///
    /// The unique identifier [`Uuid`] of the newly created document.
    #[must_use]
    pub fn create_document(&self, cb: &mut ChangeBuilder) -> DocumentUuid {
        let uuid = DocumentUuid::new_v4();

        cb.changes.push(Change::CreateDocument {
            uuid,
            name: String::new(),
        });
        uuid
    }

    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataUuid {
        let uuid = DataUuid::new_v4();
        cb.changes.push(Change::CreateData {
            module: ModuleUuid::from_module::<M>(),
            uuid,
            owner: None,
        });
        uuid
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
    pub fn open_data<M: Module>(&self, data_uuid: DataUuid) -> Option<DataView<M>> {
        // TODO: Option -> Result
        let data = &self.data.get(&data_uuid)?.downcast_ref::<M>()?;

        Some(DataView {
            project: self,
            data: data_uuid,
            persistent: &data.persistent,
            persistent_user: &data.persistent_user,
            session_data: &data.session,
            shared_data: &data.shared,
        })
    }
}
