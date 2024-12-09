//! Data and dependency management for Projects.
//!
//! Responsible for managing projects within `CADara`.
//! Projects are the primary organizational structure within `CADara`, encapsulating documents and data sections (i.e. parts and assemblies).
//! This module provides functionality to create, open, and save projects.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // we don't want 3 different `Session` types
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::missing_panics_doc)] // TODO: delete this asap

// TODO: Transactions should be split into a normal and +unchecked version

// Public modules
pub mod data;
pub mod document;
pub mod manager;
pub mod user;

use data::DataUuid;
use data::{internal::InternalData, session::internal::InternalDataSession, DataSession};
use document::{DocumentSession, DocumentUuid};
use module::Module;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use user::User;
use uuid::Uuid;

/// A trait for type-erased data models, enabling polymorphic handling of different data types.
///
/// This trait allows for the storage of any [`DataModel`] type while providing
/// mechanisms to recover the specific type through downcasting. It also facilitates
/// serialization of data without knowing their concrete types.
trait DataModelTrait: erased_serde::Serialize + Debug + Send + Any + DynClone {
    /// Retrieves a mutable reference to the underlying type as a trait object.
    /// This is used for downcasting to the concrete [`DataModel`] type.
    fn as_any(&mut self) -> &mut dyn Any;
}
dyn_clone::clone_trait_object!(DataModelTrait);
erased_serde::serialize_trait_object!(DataModelTrait);

trait SharedDataTrait: erased_serde::Serialize + Debug + Send + Any + DynClone {
    fn as_any(&mut self) -> &mut dyn Any;
}
dyn_clone::clone_trait_object!(SharedDataTrait);
erased_serde::serialize_trait_object!(SharedDataTrait);

trait SessionDataTrait: erased_serde::Serialize + Debug + Send + Any + DynClone {
    fn as_any(&mut self) -> &mut dyn Any;
}
dyn_clone::clone_trait_object!(SessionDataTrait);
erased_serde::serialize_trait_object!(SessionDataTrait);

use dyn_clone::DynClone;

trait AnyTransactionData: erased_serde::Serialize + Debug + Send + Any + DynClone {
    #[expect(dead_code)]
    fn as_any(&self) -> &dyn Any;
    /// Retrieves a mutable reference to the underlying type as a trait object.
    /// This is used for downcasting to the concrete [`TransactionData`] type.
    #[expect(dead_code)]
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

erased_serde::serialize_trait_object!(AnyTransactionData);

impl Clone for Box<dyn AnyTransactionData> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(self.as_ref())
    }
}

// TODO: WTF?
impl<T> AnyTransactionData for T
where
    T: Any + DynClone + Debug + Send + Sync + erased_serde::Serialize,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TransactionData<M: Module>(M::PersistentData);

#[derive(Clone, Debug, Deserialize)]
struct SharedDataConcrete<M: Module>(M::SharedData);

#[derive(Clone, Debug, Deserialize)]
struct SessionDataConcrete<M: Module>(M::SessionData);

impl<M: Module> SharedDataTrait for SharedDataConcrete<M> {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<M: Module> SessionDataTrait for SessionDataConcrete<M> {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// A struct for managing shared, mutable access to an [`InternalData`].
///
/// This struct encapsulates an [`InternalData`] within `Rc<RefCell<...>>` to enable
/// shared ownership and mutability across different parts of the code. It is designed to work
/// with data models that implement the [`Module`] trait.
#[derive(Clone, Debug, Deserialize)]
struct DataModel<M: Module>(Arc<Mutex<InternalData<M>>>);

#[derive(Clone, Debug, Deserialize)]
struct SharedData<M: Module>(Arc<Mutex<M::SharedData>>);

#[derive(Clone, Debug, Deserialize)]
struct SessionData<M: Module>(Arc<Mutex<M::SessionData>>);

// We use this thread local storage to pass data to the deserialize function through
// automatically derived implementations of `Deserialize`. Alternatively, we could
// replace each step of the deserialization process with a custom implementation with a seed
// that contains the registry, but this would be more complex and less maintainable.
// TODO: look into alternatives to thread local storage
thread_local! {
    static MODULE_REGISTRY: RefCell<Option<*const ModuleRegistry>> = const { RefCell::new(None) };
}

/// A struct representing a type-erased [`DataModel`].
///
/// This struct holds a [`Uuid`] identifying the document and a boxed [`DataModelTrait`],
/// allowing for the storage and serialization of various data types without
/// knowing their concrete types at compile time.
#[derive(Debug, Serialize, Clone)]
struct ErasedDataModel {
    uuid: Uuid,
    model: Box<dyn DataModelTrait>,
}

#[derive(Debug, Serialize, Clone)]
struct ErasedSharedData {
    uuid: Uuid, // Make a ModelUuid newtype
    model: Box<dyn SharedDataTrait>,
}

#[derive(Debug, Serialize)]
struct ErasedSessionData {
    uuid: Uuid,
    model: Box<dyn SessionDataTrait>, // TODO: rename to 'data', test
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum TransactionTarget {
    PersistentData(DataUuid),
    PersistendUserData(DataUuid, User),
}

// We manually implement deserialization logic to support runtime polymorphism
// The `typetag` could do this for us, but it unfortunately does not support WebAssembly
#[derive(Clone, Debug, Serialize)]
struct ErasedTransactionData {
    uuid: Uuid, // TODO: rename to indicate that this is the UUID of the module for deserailization/serialization
    target: TransactionTarget,
    data: Box<dyn AnyTransactionData>, // TODO: use smallbox::SmallBox instead of Box
}

impl<M: Module> Serialize for TransactionData<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<M: Module> Serialize for SessionDataConcrete<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<M: Module> Serialize for SharedDataConcrete<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ErasedTransactionData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Retrieve the registry from thread local storage
        // And use it to deserialize the model using the seed
        MODULE_REGISTRY.with(|r| {
            let registry = r.borrow();
            let registry = registry.expect("no registry found");
            let seed = TransactionSeed {
                // As long as the registry is alive, we can safely hold a reference to it.
                // The registry is only invalidated after deserialization is complete, so only
                // after this reference is dropped.
                registry: unsafe { &*registry },
            };
            seed.deserialize(deserializer)
        })
    }
}

/// Document in a Project
///
/// Defines the metadata and the identifiers of containing data sections.
#[derive(Debug, Serialize, Deserialize, Default)]
struct DocumentRecord {
    data: Vec<DataUuid>,
}

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
        self.0.lock().unwrap().serialize(serializer)
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

impl<'de> Deserialize<'de> for ErasedSharedData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Retrieve the registry from thread local storage
        // And use it to deserialize the model using the seed
        MODULE_REGISTRY.with(|r| {
            let registry = r.borrow();
            let registry = registry.expect("no registry found");
            let seed = ErasedSharedDataSeed {
                // As long as the registry is alive, we can safely hold a reference to it.
                // The registry is only invalidated after deserialization is complete, so only
                // after this reference is dropped.
                registry: unsafe { &*registry },
            };
            seed.deserialize(deserializer)
        })
    }
}

impl<'de> Deserialize<'de> for ErasedSessionData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Retrieve the registry from thread local storage
        // And use it to deserialize the model using the seed
        MODULE_REGISTRY.with(|r| {
            let registry = r.borrow();
            let registry = registry.expect("no registry found");
            let seed = ErasedSessionDataSeed {
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
    // TODO: remove modules,
    modules: HashMap<Uuid, BoxedDeserializeFunction<Box<dyn DataModelTrait>>>,
    modules2: HashMap<Uuid, BoxedDeserializeFunction<Box<dyn AnyTransactionData>>>,
    modules3: HashMap<Uuid, BoxedDeserializeFunction<Box<dyn SharedDataTrait>>>,
    modules4: HashMap<Uuid, BoxedDeserializeFunction<Box<dyn SessionDataTrait>>>,
}

impl ModuleRegistry {
    pub fn register<M>(&mut self)
    where
        M: Module + for<'de> Deserialize<'de>,
    {
        self.modules.insert(M::uuid(), |d| {
            Ok(Box::new(erased_serde::deserialize::<DataModel<M>>(d)?))
        });
        self.modules2.insert(M::uuid(), |d| {
            Ok(Box::new(erased_serde::deserialize::<TransactionData<M>>(
                d,
            )?))
        });
        self.modules3.insert(M::uuid(), |d| {
            Ok(Box::new(
                erased_serde::deserialize::<SharedDataConcrete<M>>(d)?,
            ))
        });
        self.modules4.insert(M::uuid(), |d| {
            Ok(Box::new(
                erased_serde::deserialize::<SessionDataConcrete<M>>(d)?,
            ))
        });
    }
}

struct ModuleSeed<'a> {
    pub registry: &'a ModuleRegistry,
}

struct ErasedSharedDataSeed<'a> {
    pub registry: &'a ModuleRegistry,
}

struct ErasedSessionDataSeed<'a> {
    pub registry: &'a ModuleRegistry,
}

struct TransactionSeed<'a> {
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

impl<'a, 'de> DeserializeSeed<'de> for ErasedSharedDataSeed<'a>
where
    'a: 'de,
{
    type Value = ErasedSharedData;

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
            marker: PhantomData<ErasedSharedData>,
            lifetime: PhantomData<&'de ()>,
            registry: &'de ModuleRegistry,
        }

        impl<'de> Visitor<'de> for ModuleVisitor<'de> {
            type Value = ErasedSharedData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ErasedSharedData")
            }

            #[inline]
            fn visit_seq<V>(self, mut _seq: V) -> Result<ErasedSharedData, V::Error>
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
                // Ok(ErasedSharedData { uuid, model })
                todo!("sequential deserialization of ErasedSharedData is not supported yet")
            }

            #[inline]
            fn visit_map<V>(self, mut map: V) -> Result<ErasedSharedData, V::Error>
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
                            let d = self.registry.modules3.get(&uuid).ok_or_else(|| {
                                serde::de::Error::custom("module not found in registry")
                            })?;

                            model = Some(map.next_value_seed(BoxedDeserializerSeed(*d))?);
                        }
                        ModuleField::Ignore => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(ErasedSharedData {
                    uuid: uuid.ok_or_else(|| serde::de::Error::missing_field("uuid"))?,
                    model: model.ok_or_else(|| serde::de::Error::missing_field("model"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["uuid", "model"];
        deserializer.deserialize_struct(
            "ErasedSharedData",
            FIELDS,
            ModuleVisitor {
                marker: PhantomData::<ErasedSharedData>,
                lifetime: PhantomData,
                registry: self.registry,
            },
        )
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ErasedSessionDataSeed<'a>
where
    'a: 'de,
{
    type Value = ErasedSessionData;

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
            marker: PhantomData<ErasedSessionData>,
            lifetime: PhantomData<&'de ()>,
            registry: &'de ModuleRegistry,
        }

        impl<'de> Visitor<'de> for ModuleVisitor<'de> {
            type Value = ErasedSessionData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ErasedSessionData")
            }

            #[inline]
            fn visit_seq<V>(self, mut _seq: V) -> Result<ErasedSessionData, V::Error>
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
                // Ok(ErasedSessionData { uuid, model })
                todo!("sequential deserialization of ErasedSessionData is not supported yet")
            }

            #[inline]
            fn visit_map<V>(self, mut map: V) -> Result<ErasedSessionData, V::Error>
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
                            let d = self.registry.modules4.get(&uuid).ok_or_else(|| {
                                serde::de::Error::custom("module not found in registry")
                            })?;

                            model = Some(map.next_value_seed(BoxedDeserializerSeed(*d))?);
                        }
                        ModuleField::Ignore => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(ErasedSessionData {
                    uuid: uuid.ok_or_else(|| serde::de::Error::missing_field("uuid"))?,
                    model: model.ok_or_else(|| serde::de::Error::missing_field("model"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["uuid", "model"];
        deserializer.deserialize_struct(
            "ErasedSessionData",
            FIELDS,
            ModuleVisitor {
                marker: PhantomData::<ErasedSessionData>,
                lifetime: PhantomData,
                registry: self.registry,
            },
        )
    }
}

impl<'a, 'de> DeserializeSeed<'de> for TransactionSeed<'a>
where
    'a: 'de,
{
    type Value = ErasedTransactionData;

    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum ModuleField {
            Uuid,
            Target,
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
                    0 => Ok(ModuleField::Uuid),
                    2 => Ok(ModuleField::Target),
                    3 => Ok(ModuleField::Data),
                    _ => Ok(ModuleField::Ignore),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "uuid" => Ok(ModuleField::Uuid),
                    "target" => Ok(ModuleField::Target),
                    "data" => Ok(ModuleField::Data),
                    _ => Ok(ModuleField::Ignore),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"uuid" => Ok(ModuleField::Uuid),
                    b"target" => Ok(ModuleField::Target),
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
            marker: PhantomData<ErasedTransactionData>,
            lifetime: PhantomData<&'de ()>,
            registry: &'de ModuleRegistry,
        }

        impl<'de> Visitor<'de> for ModuleVisitor<'de> {
            type Value = ErasedTransactionData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ErasedTransactionData")
            }

            #[inline]
            fn visit_seq<V>(self, mut _seq: V) -> Result<ErasedTransactionData, V::Error>
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
                todo!("sequential deserialization of ErasedTransactionData is not supported yet")
            }

            #[inline]
            fn visit_map<V>(self, mut map: V) -> Result<ErasedTransactionData, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut uuid = None;
                let mut target = None;
                let mut data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        ModuleField::Uuid => {
                            if uuid.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid = Some(map.next_value::<Uuid>()?);
                        }
                        ModuleField::Target => {
                            if target.is_some() {
                                return Err(serde::de::Error::duplicate_field("target"));
                            }
                            target = Some(map.next_value::<TransactionTarget>()?);
                        }
                        ModuleField::Data => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            let uuid = uuid.ok_or_else(|| {
                                serde::de::Error::custom("uuid must precede data")
                            })?;
                            let d = self.registry.modules2.get(&uuid).ok_or_else(|| {
                                serde::de::Error::custom("module not found in registry")
                            })?;

                            data = Some(map.next_value_seed(BoxedDeserializerSeed(*d))?);
                        }
                        ModuleField::Ignore => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(ErasedTransactionData {
                    uuid: uuid.ok_or_else(|| serde::de::Error::missing_field("uuid"))?,
                    target: target.ok_or_else(|| serde::de::Error::missing_field("target"))?,
                    data: data.ok_or_else(|| serde::de::Error::missing_field("data"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["uuid", "target", "data"];
        deserializer.deserialize_struct(
            "ErasedTransactionData",
            FIELDS,
            ModuleVisitor {
                marker: PhantomData::<ErasedTransactionData>,
                lifetime: PhantomData,
                registry: self.registry,
            },
        )
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum ProjectLogEntry {
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
        uuid: DataUuid,
        owner: DocumentUuid,
    },
    DeleteData {
        uuid: DataUuid,
    },
    MoveData {
        uuid: DataUuid,
        new_owner: DocumentUuid,
    },
    Transaction(ErasedTransactionData),
    // TODO: this should probably save a pointer to what to undo/redo
    Undo,
    Redo,
}

/// Represents the internal data of a `CADara` project.
///
/// This struct is used to manage the internal state of a project, including its documents (including their data),
/// name, tags, and disk path. It is not intended for direct use by consumers of the API;
/// instead, use the [`Project`] struct for public interactions.
#[derive(Serialize, Deserialize, Debug, Default)]
struct InternalProject {
    /// Chronological list of all applied [`ProjectTransaction`]s.
    log: Vec<ProjectLogEntry>,
    shared_data: HashMap<DataUuid, ErasedSharedData>,
    session_data: HashMap<DataUuid, ErasedSessionData>,
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
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Project {
    /// Encapsulates the internal representation of the project, including documents and metadata.
    project: Arc<Mutex<InternalProject>>,
}

impl Project {
    //  TODO: document
    #[must_use]
    pub fn create_view(&self) -> ProjectView {
        // TODO: no clone!
        ProjectView {
            project: self.project.clone(),
            user: User::local(),
            data: HashMap::new(),
            name: String::new(),
            tags: Vec::new(),
        }
    }

    /// Creates a new project with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the project.
    #[must_use]
    pub fn new(_name: String) -> Self {
        Self {
            project: Arc::new(Mutex::new(InternalProject::default())),
        }
    }

    /// Creates a new project given the name, user and path.
    /// TODO: replace this with a proper, maybe hide except for project manager
    #[must_use]
    pub fn new_with_path(_name: String, _user: User, _path: PathBuf) -> Self {
        todo!("remove or implement this")
    }
}

/// TODO: document
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectView {
    /// Encapsulates the internal representation of the project, including documents and metadata.
    project: Arc<Mutex<InternalProject>>,
    /// The user currently interacting with the project.
    user: User,
    /// TODO: document
    data: HashMap<DataUuid, ErasedDataModel>,
    /// TODO: document
    /// The name of the project.
    name: String,
    /// A list of tags associated with the project for categorization or searchability.
    tags: Vec<String>,
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
    pub fn open_document(&self, document_uuid: DocumentUuid) -> Option<DocumentSession> {
        let _ = self.project.lock().unwrap().documents.get(&document_uuid)?;
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
    pub fn create_document(&self) -> DocumentUuid {
        let new_doc_uuid = DocumentUuid::new_v4();

        let mut project = self.project.lock().unwrap();
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
    #[allow(clippy::significant_drop_tightening)] // This lint broken here, want's to delete a used variable
    pub fn open_data<M: Module>(&self, data_uuid: DataUuid) -> Option<DataSession<M>> {
        // TODO: Option -> Result
        let project = &self.project;

        // first, we get the document model from the project (if it exists)
        let mut mut_project = project.lock().unwrap();
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
            data_model_ref: Arc::downgrade(&data_model.0),
        })
    }
}
