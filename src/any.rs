use dyn_clone::DynClone;
use std::{
    any::{type_name, Any, TypeId},
    hash::Hash,
};

/// Conversion to [`Any`] to workaround [#65991](https://github.com/rust-lang/rust/issues/65991).
/// Implemented for anything that's `'static` and [`Clone`].
pub trait IntoAny: DynClone + Any {
    /// Type erase `self`.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

dyn_clone::clone_trait_object!(IntoAny);

impl<T: 'static + Clone> IntoAny for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }
}

/// A [`TypeId`] and the type's name.
#[derive(Debug, Clone, Copy)]
pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
}

impl TypeInfo {
    /// Gets the [`TypeId`].
    pub fn id(&self) -> TypeId {
        self.id
    }

    /// Gets the type name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Hash for TypeInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &TypeInfo) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for TypeInfo {}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &TypeInfo) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// A type with its name remembered.
/// Implemented for anything that's `'static` and [`Clone`].
pub trait NamedAny: IntoAny {
    /// Gets the type id and name from a value.
    fn type_info(&self) -> TypeInfo;
}

dyn_clone::clone_trait_object!(NamedAny);

impl<T: 'static + Clone> NamedAny for T {
    fn type_info(&self) -> TypeInfo {
        TypeInfo {
            id: self.type_id(),
            name: type_name::<T>(),
        }
    }
}

/// Type eraser of [`NamedAny`].
pub type DynAny = Box<dyn NamedAny>;

impl std::fmt::Debug for DynAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedAny").finish_non_exhaustive()
    }
}

pub fn type_info<T: 'static>() -> TypeInfo {
    TypeInfo {
        id: TypeId::of::<T>(),
        name: type_name::<T>(),
    }
}
