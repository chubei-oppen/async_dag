//! Utility structs and traits for manipulating tuples and tuple of [`Option`]s.

use std::any::{type_name, Any, TypeId};

/// Type eraser used for inserting to a [`TupleOption`].
pub type DynAny = Box<dyn Any>;

/// Type used for indexing a [`TupleOption`].
pub type TupleIndex = u8;

/// The error that can happen when inserting to a [`TupleOption`].
#[derive(Debug)]
pub struct InsertError {
    /// The error kind.
    pub kind: InsertErrorKind,
    /// The value that was inserted when this error happens.
    pub value: DynAny,
}

/// The [`InsertError`] kind.
#[derive(Debug)]
pub enum InsertErrorKind {
    /// The inserted value's type is not the expected one.
    TypeMismatch {
        /// The mismatched input's index.
        index: TupleIndex,
        /// The expected type's [`TypeId`].
        expected: TypeId,
        /// The expected type's name.
        name: &'static str,
    },
    /// The inserting index is out of range.
    OutOfRange {
        /// Length of the [`TupleOption`].
        len: TupleIndex,
        /// The inserting index.
        index: TupleIndex,
    },
}

impl std::fmt::Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertError")
            .field("kind", &self.kind)
            .field("value", &self.value.type_id())
            .finish()
    }
}

impl std::error::Error for InsertError {}

/// The result of inserting to a [`TupleOption`].
pub type InsertResult = Result<(), InsertError>;

/// The error that can happen when taking from [`TupleOption`].
#[derive(Debug)]
pub struct TakeError {
    /// The first missing input's index.
    pub index: TupleIndex,
}

impl std::fmt::Display for TakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TakeError")
            .field("index", &self.index)
            .finish()
    }
}

impl std::error::Error for TakeError {}

/// Implemented for all [`Sized`] + `'static` tuple of [`Option`]s.
pub trait TupleOption<T>: Default {
    /// Returns index of the first element that is [`None`].
    fn first_none(&self) -> Option<TupleIndex>;

    /// Inserts `value` at `index`.
    ///
    /// `self` is unchanged on error.
    fn insert(&mut self, index: TupleIndex, value: DynAny) -> InsertResult;

    /// Takes the values out.
    ///
    /// `self` is unchanged on error.
    fn take(&mut self) -> Result<T, TakeError>;
}

impl TupleOption<()> for () {
    fn first_none(&self) -> Option<TupleIndex> {
        None
    }

    fn insert(&mut self, index: TupleIndex, value: DynAny) -> InsertResult {
        Err(InsertError {
            kind: InsertErrorKind::OutOfRange { len: 0, index },
            value,
        })
    }

    fn take(&mut self) -> Result<(), TakeError> {
        Ok(())
    }
}

impl<T0: Any> TupleOption<(T0,)> for (std::option::Option<T0>,) {
    fn first_none(&self) -> Option<TupleIndex> {
        if self.0.is_none() {
            return Some(0);
        }
        None
    }

    fn insert(&mut self, index: TupleIndex, value: DynAny) -> InsertResult {
        match index {
            0 => match Box::<dyn Any>::downcast::<T0>(value) {
                Ok(t) => self.0 = Some(*t),
                Err(value) => {
                    return Err(InsertError {
                        kind: InsertErrorKind::TypeMismatch {
                            index: 0,
                            expected: TypeId::of::<T0>(),
                            name: type_name::<T0>(),
                        },
                        value,
                    });
                }
            },
            _ => {
                return Err(InsertError {
                    kind: InsertErrorKind::OutOfRange { len: 1, index },
                    value,
                })
            }
        }
        Ok(())
    }

    fn take(&mut self) -> Result<(T0,), TakeError> {
        match self.first_none() {
            Some(index) => Err(TakeError { index }),
            None => Ok((self.0.take().unwrap(),)),
        }
    }
}

impl<T0: Any, T1: Any> TupleOption<(T0, T1)>
    for (std::option::Option<T0>, std::option::Option<T1>)
{
    fn first_none(&self) -> Option<TupleIndex> {
        if self.0.is_none() {
            return Some(0);
        }
        if self.1.is_none() {
            return Some(1);
        }
        None
    }

    fn insert(&mut self, index: TupleIndex, value: DynAny) -> InsertResult {
        match index {
            0 => match Box::<dyn Any>::downcast::<T0>(value) {
                Ok(t) => self.0 = Some(*t),
                Err(value) => {
                    return Err(InsertError {
                        kind: InsertErrorKind::TypeMismatch {
                            index: 0,
                            expected: TypeId::of::<T0>(),
                            name: type_name::<T0>(),
                        },
                        value,
                    });
                }
            },
            1 => match Box::<dyn Any>::downcast::<T1>(value) {
                Ok(t) => self.1 = Some(*t),
                Err(value) => {
                    return Err(InsertError {
                        kind: InsertErrorKind::TypeMismatch {
                            index: 1,
                            expected: TypeId::of::<T1>(),
                            name: type_name::<T1>(),
                        },
                        value,
                    });
                }
            },
            _ => {
                return Err(InsertError {
                    kind: InsertErrorKind::OutOfRange { len: 1, index },
                    value,
                })
            }
        }
        Ok(())
    }

    fn take(&mut self) -> Result<(T0, T1), TakeError> {
        match self.first_none() {
            Some(index) => Err(TakeError { index }),
            None => Ok((self.0.take().unwrap(), self.1.take().unwrap())),
        }
    }
}

/// Implemented for all [`Sized`] + `'static` tuples.
pub trait Tuple: Sized {
    type Option: TupleOption<Self>;
}

impl Tuple for () {
    type Option = ();
}

impl<T0: Any> Tuple for (T0,) {
    type Option = (std::option::Option<T0>,);
}

impl<T0: Any, T1: Any> Tuple for (T0, T1) {
    type Option = (std::option::Option<T0>, std::option::Option<T1>);
}
