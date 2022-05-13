//! Utility structs and traits for manipulating tuples and tuple of [`Option`]s.

use crate::any::DynAny;
use crate::any::TypeInfo;
use std::any::{type_name, Any, TypeId};

/// Type used for indexing a [`TupleOption`].
pub type TupleIndex = u8;

/// The error that can happen when inserting to a [`TupleOption`].
#[derive(Debug)]
pub struct InsertError {
    /// The error kind.
    pub kind: InsertErrorKind,
    /// The value that was inserted when this error happens.
    pub value: Box<dyn Any>,
}

/// The [`InsertError`] kind.
#[derive(Debug)]
pub enum InsertErrorKind {
    /// The inserted value's type is not the expected one.
    TypeMismatch {
        /// The expected type's [`TypeId`].
        expected: TypeId,
        /// The expected type's name.
        expected_name: &'static str,
    },
    /// The inserting index is out of range.
    OutOfRange,
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
pub trait TupleOption<T: Tuple>: Default {
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
        #[allow(clippy::match_single_binding)]
        match index {
            _ => Err(InsertError {
                kind: InsertErrorKind::OutOfRange,
                value: value.into_any(),
            }),
        }
    }

    fn take(&mut self) -> Result<(), TakeError> {
        match self.first_none() {
            Some(index) => Err(TakeError { index }),
            None => Ok(()),
        }
    }
}

impl<T0: Any> TupleOption<(T0,)> for (Option<T0>,) {
    fn first_none(&self) -> Option<TupleIndex> {
        if self.0.is_none() {
            return Some(0);
        }
        None
    }

    fn insert(&mut self, index: TupleIndex, value: DynAny) -> InsertResult {
        #[allow(clippy::match_single_binding)]
        match index {
            0 => match Box::<dyn Any>::downcast::<T0>(value.into_any()) {
                Ok(t) => {
                    self.0 = Some(*t);
                    Ok(())
                }
                Err(value) => Err(InsertError {
                    kind: InsertErrorKind::TypeMismatch {
                        expected: TypeId::of::<T0>(),
                        expected_name: type_name::<T0>(),
                    },
                    value,
                }),
            },
            _ => Err(InsertError {
                kind: InsertErrorKind::OutOfRange,
                value: value.into_any(),
            }),
        }
    }

    fn take(&mut self) -> Result<(T0,), TakeError> {
        match self.first_none() {
            Some(index) => Err(TakeError { index }),
            None => Ok((self.0.take().unwrap(),)),
        }
    }
}

impl<T0: Any, T1: Any> TupleOption<(T0, T1)> for (Option<T0>, Option<T1>) {
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
        #[allow(clippy::match_single_binding)]
        match index {
            0 => match Box::<dyn Any>::downcast::<T0>(value.into_any()) {
                Ok(t) => self.0 = Some(*t),
                Err(value) => {
                    return Err(InsertError {
                        kind: InsertErrorKind::TypeMismatch {
                            expected: TypeId::of::<T0>(),
                            expected_name: type_name::<T0>(),
                        },
                        value,
                    });
                }
            },
            1 => match Box::<dyn Any>::downcast::<T1>(value.into_any()) {
                Ok(t) => self.1 = Some(*t),
                Err(value) => {
                    return Err(InsertError {
                        kind: InsertErrorKind::TypeMismatch {
                            expected: TypeId::of::<T1>(),
                            expected_name: type_name::<T1>(),
                        },
                        value,
                    });
                }
            },
            _ => {
                return Err(InsertError {
                    kind: InsertErrorKind::OutOfRange,
                    value: value.into_any(),
                })
            }
        }
        #[allow(unreachable_code)]
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
    /// The corresponding tuple of [`Option`]s.
    type Option: TupleOption<Self>;

    /// Length of the tuple.
    const LEN: TupleIndex;

    /// [`TypeId`] and name of the type at `index`.
    ///
    /// Returns [`None`] if `index` is out of range.
    fn type_info(index: TupleIndex) -> Option<TypeInfo>;
}

impl Tuple for () {
    type Option = ();

    const LEN: TupleIndex = 0;

    fn type_info(index: TupleIndex) -> Option<TypeInfo> {
        #[allow(clippy::match_single_binding)]
        match index {
            _ => None,
        }
    }
}

impl<T0: Any> Tuple for (T0,) {
    type Option = (Option<T0>,);

    const LEN: TupleIndex = 1;

    fn type_info(index: TupleIndex) -> Option<TypeInfo> {
        #[allow(clippy::match_single_binding)]
        match index {
            0 => Some(TypeInfo::of::<T0>()),
            _ => None,
        }
    }
}

impl<T0: Any, T1: Any> Tuple for (T0, T1) {
    type Option = (Option<T0>, Option<T1>);

    const LEN: TupleIndex = 2;

    fn type_info(index: TupleIndex) -> Option<TypeInfo> {
        #[allow(clippy::match_single_binding)]
        match index {
            0 => Some(TypeInfo::of::<T0>()),
            1 => Some(TypeInfo::of::<T1>()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mismatch_type_name() {
        let mut option: (Option<i32>,) = (None,);
        let error = option.insert(0, Box::new(0.0f32)).unwrap_err();
        let expected_name = match error.kind {
            InsertErrorKind::TypeMismatch { expected_name, .. } => expected_name,
            _ => panic!("Expecting TypeMismatch"),
        };
        assert!(expected_name.contains("i32"));
    }
}
