use dyn_clone::DynClone;
use std::any::{type_name, Any};

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

/// A type with its name. Implemented for anything that's `'static` and [`Clone`].
pub trait NamedAny: IntoAny {
    /// Gets the type name from a value.
    fn type_name(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(NamedAny);

impl<T: 'static + Clone> NamedAny for T {
    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

/// Type eraser of [`NamedAny`].
pub type DynAny = Box<dyn NamedAny>;
