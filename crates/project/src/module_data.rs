//! Type erasure and serialization support for data types generic of [`Module`].
//!
//! Provides machinery for:
//! - Type-erased data containers for module-specific data
//! - Serialization/deserialization of type-erased data
//! - Module registry for runtime type information

use core::fmt;
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
use uuid::Uuid;

/// Globally unique identifier of a [`Module`].
///
/// Newtype around [`Module::uuid`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ModuleUuid(Uuid);

impl ModuleUuid {
    /// Create a [`ModuleUuid`] from a [`Module`].
    pub fn from_module<M: Module>() -> Self {
        Self(M::uuid())
    }
}

impl fmt::Display for ModuleUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModuleUuid({})", self.0)
    }
}

/// Generates type-erased implementations for a data type that is generic over [`Module`].
///
/// This macro creates a complete type erasure solution by implementing:
/// - Serialization and deserialization support
/// - Dynamic wrapper types with safe downcasting capabilities
/// - Clone functionality for dynamic types
///
/// Using [`serde::Deserialize`] only works when inside a [`Project`] and using [`ProjectDeserializer`]
/// since [`MODULE_REGISTRY`] is accessed.
///
/// # Arguments
/// * `$d` - The data type to implement type erasure for
/// * `$reg_entry` - Field name in [`ModuleRegEntry`] containing the deserializer function.
///    The field must be of type `BoxedDeserializeFunction<Box<dyn dTrait>>` where `d` is
///    the name of the type passed into this macro.
///
/// # Generated Types
/// For an input type `T`, the macro generates:
/// - `TTrait` - Common behavior trait for type-erased operations
/// - `DynT` - Type-erased wrapper with serialization capabilities
/// - `TDeserializer` - Deserializer for type-erased data
#[macro_export] // TODO: make private
macro_rules! define_type_erased_data {
    ($d:ty, $reg_entry:ident) => {
        paste! {
            #[doc = "A trait shared by all [`" $d "`] types for all [`Module`]"]
            #[allow(dead_code)]
            pub trait [<$d Trait>]: erased_serde::Serialize + Debug + Send + Sync + Any + DynClone {
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
            pub struct [<Dyn $d>] {
                // uuid of the module, over that the struct contained in `data` is generic
                pub module: ModuleUuid,
                #[doc = "Type erased [`" $d "`]"]
                pub data: Box<dyn [<$d Trait>]>,
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
                pub registry: &'a ModuleRegistry,
            }

            // We manually implement deserialization logic to support runtime polymorphism
            // The `typetag` could do this for us, but it unfortunately does not support WebAssembly
            impl<'a, 'de> DeserializeSeed<'de> for [<$d Deserializer>]<'a>
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

                                        data = Some(map.next_value_seed(BoxedDeserializeSeed(d))?);
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

/// Complete state of the data of a module, publicly accessible through a [`DataView`].
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Data<M: Module> {
    pub persistent: M::PersistentData,
    pub persistent_user: M::PersistentUserData,
    pub session: M::SessionData,
    pub shared: M::SharedData,
}
define_type_erased_data!(Data, deserialize_data);

/// Wrapper type around [`Module::SharedData`]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SharedData<M: Module>(M::SharedData);
define_type_erased_data!(SharedData, deserialize_shared);

/// Wrapper type around [`Module::SessionData`]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SessionData<M: Module>(M::SessionData);
define_type_erased_data!(SessionData, deserialize_session);

/// Wrapper type for transaction arguments that can be applied to a [`Module::PersistentData`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionData<M: Module>(pub <M::PersistentData as DataTransaction>::Args);
define_type_erased_data!(TransactionData, deserialize_transaction);

/// Type alias for a boxed deserialization function that can handle type-erased data.
///
/// # Type Parameters
/// * `O` - The output type that will be produced by deserialization
type BoxedDeserializeFunction<O> =
    for<'de> fn(&mut dyn erased_serde::Deserializer<'de>) -> Result<O, erased_serde::Error>;

/// Seed type for deserializing boxed trait objects.
///
/// Provides the machinery needed to deserialize type-erased data using a boxed
/// deserialization function.
///
/// # Type Parameters
/// * `O` - The trait object type to be deserialized
struct BoxedDeserializeSeed<O: ?Sized>(pub BoxedDeserializeFunction<Box<O>>);

impl<'de, O: ?Sized> DeserializeSeed<'de> for BoxedDeserializeSeed<O> {
    type Value = Box<O>;

    /// Deserializes a value using the contained boxed deserializer function.
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.0(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(serde::de::Error::custom)
    }
}

/// Registry entry containing type-erased functions for module specific operations.
#[derive(Clone, Debug)]
pub struct ModuleRegEntry {
    pub deserialize_data: BoxedDeserializeFunction<Box<dyn DataTrait>>,
    pub deserialize_transaction: BoxedDeserializeFunction<Box<dyn TransactionDataTrait>>,
    pub deserialize_shared: BoxedDeserializeFunction<Box<dyn SharedDataTrait>>,
    pub deserialize_session: BoxedDeserializeFunction<Box<dyn SessionDataTrait>>,
    /// Creates a new instance of type-erased module data
    pub init_data: fn() -> Box<dyn DataTrait>,
    /// Applies a type-erased transaction to type-erased module data
    pub apply_transaction: fn(&mut Box<dyn DataTrait>, &Box<dyn TransactionDataTrait>),
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
    /// Since Deserialization is single threaded and we reset the containing pointer after Deserialization,
    /// it is impossible to cause UB.
    pub static MODULE_REGISTRY: RefCell<Option<*const ModuleRegistry>> = const { RefCell::new(None) };
}

/// A registry containing all supported modules necessary for working with [`Project`]s
#[derive(Clone, Debug, Default)]
pub struct ModuleRegistry(pub HashMap<ModuleUuid, ModuleRegEntry>);

impl ModuleRegistry {
    /// Create a new, empty [`ModuleRegistry`].
    ///
    /// Use [`ModuleRegistry::register`] to add [`Module`]s to it.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a [`Module`] to be known by the [`ModuleRegistry`]
    pub fn register<M>(&mut self)
    where
        M: Module,
        M::PersistentData: for<'de> Deserialize<'de>,
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
                    let t = t
                        .as_ref()
                        .as_any()
                        .downcast_ref::<TransactionData<M>>()
                        .unwrap();
                    module::DataTransaction::apply(&mut m.persistent, t.0.clone()).unwrap();
                },
            },
        );
    }
}
