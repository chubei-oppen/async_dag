//! The error types.

use super::NodeIndex;
use crate::any::TypeInfo;
use crate::tuple::TupleIndex;

/// Errors that can happen during graph construction.
#[derive(Debug)]
pub enum Error {
    /// The specified dependent node has started running its task and can't have its dependency modified.
    HasStarted(NodeIndex),
    /// The specified dependency index is greater than or equal to the dependent node's task's number of inputs.
    OutOfRange(TupleIndex),
    /// The dependent node's task has `input` type at specified index, but the depended node's task has a different `output` type.
    TypeMismatch { input: TypeInfo, output: TypeInfo },
    /// Adding the specified dependency would have caused the graph to cycle.
    WouldCycle,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HasStarted(index) => f.debug_tuple("Error::HasStarted").field(index).finish(),
            Self::OutOfRange(len) => f.debug_tuple("Error::OutOfRange").field(len).finish(),
            Self::TypeMismatch { input, output } => f
                .debug_struct("Error::TypeMismatch")
                .field("input", input)
                .field("output", output)
                .finish(),
            Self::WouldCycle => f.debug_tuple("Error::WouldCycle").finish(),
        }
    }
}

impl std::error::Error for Error {}

/// An [`Error`] and a [`TryTask`](crate::task::TryTask).
#[derive(Debug)]
pub struct ErrorWithTask<T> {
    /// The error.
    pub error: Error,
    /// The task.
    pub task: T,
}

impl<T: std::fmt::Debug> std::fmt::Display for ErrorWithTask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorWithTask")
            .field("error", &self.error)
            .field("task", &self.task)
            .finish()
    }
}

impl<T: std::fmt::Debug> std::error::Error for ErrorWithTask<T> {}
