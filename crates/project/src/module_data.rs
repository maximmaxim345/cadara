//! Type erasure and serialization support for data types generic of [`Module`].
//!
//! Provides machinery for:
//! - Type-erased data containers for module-specific data
//! - Serialization/deserialization of type-erased data
//! - Module registry for runtime type information

use core::fmt;
use dyn_clone::DynClone;
use module::{DataSection, Module};
use paste::paste;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use thiserror::Error;
use uuid::Uuid;

/// Globally unique identifier of a [`Module`].
///
/// Newtype around [`Module::uuid`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ModuleId(Uuid);

impl ModuleId {
    /// Create a [`ModuleId`] from a [`Module`].
    pub fn from_module<M: Module>() -> Self {
        Self(M::uuid())
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModuleId({})", self.0)
    }
}

/// Generates type-erased implementations for a data type that is generic over [`Module`].
///
/// This macro creates a complete type erasure solution by implementing:
/// - Serialization and deserialization support
/// - Dynamic wrapper types with safe downcasting capabilities
/// - Clone functionality for dynamic types
///
/// Using [`serde::Deserialize`] only works when inside a [`crate::Project`] and using [`crate::ProjectDeserializer`]
/// since [`MODULE_REGISTRY`] is accessed.
///
/// # Arguments
/// * `$d` - The data type to implement type erasure for
/// * `$reg_entry` - Field name in [`ModuleRegistryEntry`] containing the deserializer function.
///    The field must be of type `BoxedDeserializeFunction<Box<dyn dTrait>>` where `d` is
///    the name of the type passed into this macro.
///
/// # Generated Types
/// For an input type `T`, the macro generates:
/// - `TTrait` - Common behavior trait for type-erased operations
/// - `DynT` - Type-erased wrapper with serialization capabilities
/// - `TDeserializer` - Deserializer for type-erased data
macro_rules! define_type_erased {
    ($d:ty, $reg_entry:ident) => {
        define_type_erased!($d, $reg_entry,);
    };
    ($d:ty, $reg_entry:ident, $($extra_traits:path),*) => {
        paste! {
            #[doc = "A trait shared by all [`" $d "`] types for all [`Module`]"]
            #[allow(dead_code)]
            pub trait [<$d Trait>]: erased_serde::Serialize + Debug + Send + Sync + Any + DynClone $(+ $extra_traits)* {
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
            pub struct [<Erased $d>] {
                // globally unique identifier of the module, over that the struct contained in `data` is generic
                pub(crate) module: ModuleId,
                #[doc = "Type erased [`" $d "`]"]
                pub(crate) data: Box<dyn [<$d Trait>]>,
            }

            #[allow(dead_code)]
            impl [<Erased $d>] {
                pub(crate) fn downcast_ref<M: Module>(&self) -> Option<&$d<M>> {
                    self.data.as_any().downcast_ref()
                }

                pub(crate) fn downcast_mut<M: Module>(&mut self) -> Option<&mut $d<M>> {
                    self.data.as_mut_any().downcast_mut()
                }

            }

            impl<M: Module> From<$d<M>> for [<Erased $d>] {
                fn from(d: $d<M>) -> Self {
                    Self {
                        module: ModuleId::from_module::<M>(),
                        data: Box::new(d),
                    }
                }
            }

            impl<'de> Deserialize<'de> for [<Erased $d>] {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    // Retrieve the registry from thread local storage
                    // And use it to deserialize the model using the seed
                    MODULE_REGISTRY.with(|r| {
                        let registry = r.borrow();
                        let registry = registry.ok_or_else(|| {
                            serde::de::Error::custom("no module registry found in thread local storage")
                        })?;
                        let seed = [<$d Deserializer>] {
                            // SAFETY: As long as the registry is alive, we can safely hold a reference to it.
                            // The registry is only invalidated after deserialization is complete, so only
                            // after this reference is dropped.
                            registry: unsafe { &*registry },
                        };
                        seed.deserialize(deserializer)
                    })
                }
            }

            struct [<$d Deserializer>]<'a> {
                pub(crate) registry: &'a ModuleRegistry,
            }

            // We manually implement deserialization logic to support runtime polymorphism
            // The `typetag` could do this for us, but it unfortunately does not support WebAssembly
            impl<'a, 'de> DeserializeSeed<'de> for [<$d Deserializer>]<'a>
            where
                'a: 'de,
            {
                type Value = [<Erased $d>];

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
                        marker: PhantomData<[<Erased $d>]>,
                        lifetime: PhantomData<&'de ()>,
                        registry: &'de ModuleRegistry,
                    }

                    impl<'de> Visitor<'de> for ModuleVisitor<'de> {
                        type Value = [<Erased $d>];

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(concat!("struct ", stringify!([<Erased $d>])))
                        }

                        #[inline]
                        fn visit_seq<V>(self, mut _seq: V) -> Result<[<Erased $d>], V::Error>
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
                        fn visit_map<V>(self, mut map: V) -> Result<[<Erased $d>], V::Error>
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
                                        module = Some(map.next_value::<ModuleId>()?);
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

                                        data = Some(map.next_value_seed(ErasedDeserializeSeed(d))?);
                                    }
                                    ModuleField::Ignore => {
                                        let _: serde::de::IgnoredAny = map.next_value()?;
                                    }
                                }
                            }
                            Ok([<Erased $d>] {
                                module: module.ok_or_else(|| serde::de::Error::missing_field("module"))?,
                                data: data.ok_or_else(|| serde::de::Error::missing_field("data"))?,
                            })
                        }
                    }

                    const FIELDS: &[&str] = &["module", "data"];
                    deserializer.deserialize_struct(
                        stringify!([<Erased $d>]),
                        FIELDS,
                        ModuleVisitor {
                            marker: PhantomData::<[<Erased $d>]>,
                            lifetime: PhantomData,
                            registry: self.registry,
                        },
                    )
                }
            }
        }
    };
}

pub trait DataCompare {
    fn persistent_eq(&self, other: &dyn DataTrait) -> bool;
    fn persistent_user_eq(&self, other: &dyn DataTrait) -> bool;
    fn session_eq(&self, other: &dyn DataTrait) -> bool;
    fn shared_eq(&self, other: &dyn DataTrait) -> bool;
}

/// Complete state of the data of a module, publicly accessible through a [`crate::DataView`].
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Data<M: Module> {
    pub persistent: M::PersistentData,
    pub persistent_user: M::PersistentUserData,
    pub session: M::SessionData,
    pub shared: M::SharedData,
}
define_type_erased!(Data, deserialize_data, DataCompare);

impl<M: Module> DataCompare for Data<M> {
    fn persistent_eq(&self, other: &dyn DataTrait) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.persistent == other.persistent)
    }
    fn persistent_user_eq(&self, other: &dyn DataTrait) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.persistent_user == other.persistent_user)
    }
    fn session_eq(&self, other: &dyn DataTrait) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.session == other.session)
    }
    fn shared_eq(&self, other: &dyn DataTrait) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.shared == other.shared)
    }
}

/// Wrapper type around [`Module::SharedData`]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SharedData<M: Module>(M::SharedData);
define_type_erased!(SharedData, deserialize_shared);

/// Wrapper type around [`Module::SessionData`]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SessionData<M: Module>(M::SessionData);
define_type_erased!(SessionData, deserialize_session);

/// Wrapper type for transaction arguments that can be applied to a [`Module::PersistentData`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTransactionArgs<M: Module>(pub <M::PersistentData as DataSection>::Args);
define_type_erased!(DataTransactionArgs, deserialize_transaction_args);

/// Wrapper type for transaction arguments that can be applied to a [`Module::PersistentData`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDataTransactionArgs<M: Module>(pub <M::PersistentUserData as DataSection>::Args);
define_type_erased!(UserDataTransactionArgs, deserialize_user_transaction_args);

/// Wrapper type for transaction arguments that can be applied to a [`Module::SessionData`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDataTransactionArgs<M: Module>(pub <M::SessionData as DataSection>::Args);
define_type_erased!(
    SessionDataTransactionArgs,
    deserialize_session_transaction_args
);

/// Wrapper type for transaction arguments that can be applied to a [`Module::SharedData`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedDataTransactionArgs<M: Module>(pub <M::SharedData as DataSection>::Args);
define_type_erased!(
    SharedDataTransactionArgs,
    deserialize_shared_transaction_args
);

/// Type alias for a function that dynamically deserializes type-erased data.
///
/// This function takes a deserializer and returns a result containing the output type.
///
/// # Type Parameters
/// * `O` - The output type that will be produced by deserialization.
type ErasedDeserializeFn<O> =
    for<'de> fn(&mut dyn erased_serde::Deserializer<'de>) -> Result<O, erased_serde::Error>;

/// A seed type for deserializing a boxed trait object.
///
/// This struct provides the necessary machinery to deserialize type-erased data that is stored
/// as a boxed trait object. It uses a given deserialization function to perform the deserialization.
///
/// # Type Parameters
/// * `O` - The trait object type to be deserialized.
struct ErasedDeserializeSeed<O: ?Sized>(pub ErasedDeserializeFn<Box<O>>);

impl<'de, O: ?Sized> DeserializeSeed<'de> for ErasedDeserializeSeed<O> {
    type Value = Box<O>;

    /// Deserializes a value using the contained boxed deserializer function.
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.0(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error)]
pub enum ModuleRegistryError {
    #[error("Failed to downcast module data")]
    DataDowncastError,
    #[error("Failed to downcast transaction arguments")]
    ArgsDowncastError,
}

/// Registry entry containing type-erased functions for module specific operations.
///
/// This is effectively a v-table for runtime polymorphism.
#[derive(Clone, Debug)]
pub struct ModuleRegistryEntry {
    pub(crate) deserialize_data: ErasedDeserializeFn<Box<dyn DataTrait>>,
    pub(crate) deserialize_transaction_args: ErasedDeserializeFn<Box<dyn DataTransactionArgsTrait>>,
    pub(crate) deserialize_user_transaction_args:
        ErasedDeserializeFn<Box<dyn UserDataTransactionArgsTrait>>,
    pub(crate) deserialize_session_transaction_args:
        ErasedDeserializeFn<Box<dyn SessionDataTransactionArgsTrait>>,
    pub(crate) deserialize_shared_transaction_args:
        ErasedDeserializeFn<Box<dyn SharedDataTransactionArgsTrait>>,
    pub(crate) deserialize_shared: ErasedDeserializeFn<Box<dyn SharedDataTrait>>,
    pub(crate) deserialize_session: ErasedDeserializeFn<Box<dyn SessionDataTrait>>,
    /// Creates a new instance of type-erased module data
    pub(crate) init_data: fn() -> ErasedData,
    /// Creates a new instance of type-erased session data
    pub(crate) init_session_data: fn() -> ErasedSessionData,
    /// Creates a new instance of type-erased shared data
    pub(crate) init_shared_data: fn() -> ErasedSharedData,
    /// Applies a type-erased transaction to [`Module::PersistentData`].
    pub(crate) apply_data_transaction:
        fn(&mut ErasedData, &ErasedDataTransactionArgs) -> Result<(), ModuleRegistryError>,
    /// Overrides [`Data::session`] with the given [`SessionData`]
    pub(crate) replace_session_data:
        fn(&mut ErasedData, &ErasedSessionData) -> Result<(), ModuleRegistryError>,
    /// Overrides [`Data::shared`] with the given [`SharedData`]
    pub(crate) replace_shared_data:
        fn(&mut ErasedData, &ErasedSharedData) -> Result<(), ModuleRegistryError>,
    /// Applies a type-erased transaction to [`Module::PersistentUserData`].
    pub(crate) apply_user_data_transaction:
        fn(&mut ErasedData, &ErasedUserDataTransactionArgs) -> Result<(), ModuleRegistryError>,
    /// Applies a type-erased transaction to [`Module::SessionData`].
    pub(crate) apply_session_data_transaction: fn(
        &mut ErasedSessionData,
        &ErasedSessionDataTransactionArgs,
    ) -> Result<(), ModuleRegistryError>,
    /// Applies a type-erased transaction to [`Module::SharedData`].
    pub(crate) apply_shared_data_transaction: fn(
        &mut ErasedSharedData,
        &ErasedSharedDataTransactionArgs,
    ) -> Result<(), ModuleRegistryError>,
}

thread_local! {
    /// Thread local storage of a [`ModuleRegistry`].
    ///
    /// Thread local storage to pass a [`ModuleRegistry`] to the custom implementations of [`serde::Deserialize`] through
    /// automatically derived implementations.
    ///
    /// Alternatively, we could replace each step of the deserialization process with a custom implementation with a seed
    /// that contains the registry, but this would be considerably more complex and less maintainable.
    ///
    /// # Safety
    ///
    /// Since Deserialization is single threaded, and we reset the containing pointer after Deserialization,
    /// it is impossible to cause UB.
    pub static MODULE_REGISTRY: RefCell<Option<*const ModuleRegistry>> = const { RefCell::new(None) };
}

/// A registry containing all supported modules necessary for working with a [`crate::Project`].
///
/// By default, the [`ModuleRegistry`] is empty. You must first register [`Module`]s with [`ModuleRegistry::register`].
#[derive(Clone, Debug, Default)]
pub struct ModuleRegistry(pub(crate) HashMap<ModuleId, ModuleRegistryEntry>);

impl ModuleRegistry {
    /// Create a new, empty [`ModuleRegistry`].
    ///
    /// Use [`ModuleRegistry::register`] to add [`Module`]s to it.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a [`Module`] to be known by the [`ModuleRegistry`]
    #[expect(clippy::too_many_lines)]
    pub fn register<M>(&mut self)
    where
        M: Module,
        M::PersistentData: for<'de> Deserialize<'de>,
    {
        // TODO: these are a lot of functions, replace_session_data and replace_shared_data should
        // probably be replaced with a better system
        self.0.insert(
            ModuleId::from_module::<M>(),
            ModuleRegistryEntry {
                deserialize_data: |d| Ok(Box::new(erased_serde::deserialize::<Data<M>>(d)?)),
                deserialize_transaction_args: |d| {
                    Ok(Box::new(
                        erased_serde::deserialize::<DataTransactionArgs<M>>(d)?,
                    ))
                },
                deserialize_user_transaction_args: |d| {
                    Ok(Box::new(erased_serde::deserialize::<
                        UserDataTransactionArgs<M>,
                    >(d)?))
                },
                deserialize_session_transaction_args: |d| {
                    Ok(Box::new(erased_serde::deserialize::<
                        SessionDataTransactionArgs<M>,
                    >(d)?))
                },
                deserialize_shared_transaction_args: |d| {
                    Ok(Box::new(erased_serde::deserialize::<
                        SharedDataTransactionArgs<M>,
                    >(d)?))
                },
                deserialize_shared: |d| {
                    Ok(Box::new(erased_serde::deserialize::<SharedData<M>>(d)?))
                },
                deserialize_session: |d| {
                    Ok(Box::new(erased_serde::deserialize::<SessionData<M>>(d)?))
                },
                init_data: || ErasedData {
                    module: ModuleId::from_module::<M>(),
                    data: Box::new(Data::<M>::default()),
                },
                init_session_data: || ErasedSessionData {
                    module: ModuleId::from_module::<M>(),
                    data: Box::new(SessionData::<M>::default()),
                },
                init_shared_data: || ErasedSharedData {
                    module: ModuleId::from_module::<M>(),
                    data: Box::new(SharedData::<M>::default()),
                },
                apply_data_transaction: |data, args| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let args = args
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    DataSection::apply(&mut data.persistent, args.0.clone());
                    Ok(())
                },
                replace_session_data: |data, session_data| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let session_data = session_data
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    data.session = session_data.0.clone();
                    Ok(())
                },
                replace_shared_data: |data, shared_data| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let shared_data = shared_data
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    data.shared = shared_data.0.clone();
                    Ok(())
                },
                apply_user_data_transaction: |data, args| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let args = args
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    DataSection::apply(&mut data.persistent_user, args.0.clone());
                    Ok(())
                },
                apply_session_data_transaction: |data, args| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let args = args
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    DataSection::apply(&mut data.0, args.0.clone());
                    Ok(())
                },
                apply_shared_data_transaction: |data, args| {
                    let data = data
                        .downcast_mut::<M>()
                        .ok_or(ModuleRegistryError::DataDowncastError)?;
                    let args = args
                        .downcast_ref::<M>()
                        .ok_or(ModuleRegistryError::ArgsDowncastError)?;
                    DataSection::apply(&mut data.0, args.0.clone());
                    Ok(())
                },
            },
        );
    }
}
