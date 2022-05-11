use crate::task::Message;
use crate::task::Task;
use crate::tuple::InsertResult;
use crate::tuple::TakeError;
use crate::tuple::Tuple;
use crate::tuple::TupleOption;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::any::Any;

pub type DynMessage = Box<dyn Message>;
pub type TaskFuture<'a> = BoxFuture<'a, DynMessage>;

/// [`Curry`] describes the process of currying and finally calling.
pub trait Curry<'a> {
    /// If the inner task's inputs has been populated and becomes ready for running.
    fn ready(&self) -> bool;

    /// Inserts a input to the inner task, i.e. currying.
    ///
    /// `self` is unchanged on error.
    fn curry(&mut self, index: u8, value: Box<dyn Any>) -> InsertResult;

    /// Consumes the inner task and inputs and returns a future of the output value.
    fn call(self: Box<Self>) -> Result<TaskFuture<'a>, TakeError>;
}

/// [`CurriedTask`] holds a task and its inputs and tracks if all inputs are ready.
pub struct CurriedTask<'a, T: Task<'a>> {
    task: T,
    inputs: <T::Inputs as Tuple>::Option,
}

impl<'a, T: Task<'a>> CurriedTask<'a, T> {
    /// Creates a [CurriedTask] from a task and no inputs.
    pub fn new(task: T) -> Self {
        CurriedTask {
            task,
            inputs: Default::default(),
        }
    }
}

fn make_any<T: Message>(t: T) -> DynMessage {
    Box::new(t)
}

impl<'a, T: Task<'a>> Curry<'a> for CurriedTask<'a, T> {
    fn ready(&self) -> bool {
        self.inputs.first_none().is_none()
    }

    fn curry(&mut self, index: u8, value: Box<dyn Any>) -> InsertResult {
        self.inputs.insert(index, value)
    }

    fn call(self: Box<Self>) -> Result<BoxFuture<'a, DynMessage>, TakeError> {
        let CurriedTask { task, mut inputs } = *self;
        let inputs = inputs.take()?;
        let future = task.run(inputs);
        let future = future.map(make_any);
        Ok(future.boxed())
    }
}
