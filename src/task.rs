use crate::tuple::Tuple;
use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;

/// Outpus of the [`Task`]s. Implemented for anything that's `'static` and [`Clone`].
pub trait Message: 'static {
    /// Clone and type erase `self`.
    fn clone_any(&self) -> Box<dyn Any>;

    /// Type erase `self`.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: 'static + Clone> Message for T {
    fn clone_any(&self) -> Box<dyn Any> {
        Box::new(self.clone())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }
}

/// An async task.
pub trait Task<'a> {
    /// Tuple of inputs.
    type Inputs: Tuple;

    /// Output.
    type Output: Message;

    /// Output future.
    type Future: Future<Output = Self::Output> + Send + 'a;

    /// Runs the task and gets a future.
    fn run(self, inputs: Self::Inputs) -> Self::Future;
}

/// Conversion into a [`Task`].
pub trait IntoTask<'a, Args, Out> {
    /// The [`Task`] type.
    type Task: Task<'a, Output = Out> + 'a;

    /// The conversion.
    fn into_task(self) -> Self::Task;
}

impl<'a, Fn, Out, Fut> IntoTask<'a, (), Out> for Fn
where
    Fn: FnOnce() -> Fut + 'a,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
{
    type Task = FnOnceTask<Fn, Fut, ()>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Out, Fut, I1> IntoTask<'a, (I1,), Out> for Fn
where
    Fn: FnOnce(I1) -> Fut + 'a,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
    I1: Message,
{
    type Task = FnOnceTask<Fn, Fut, (I1,)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Out, Fut, I1, I2> IntoTask<'a, (I1, I2), Out> for Fn
where
    Fn: FnOnce(I1, I2) -> Fut + 'a,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
    I1: Message,
    I2: Message,
{
    type Task = FnOnceTask<Fn, Fut, (I1, I2)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

/// A [Task] for types that implement [FnOnce].
pub struct FnOnceTask<Fn, Fut, Args> {
    function: Fn,
    fut: PhantomData<Fut>,
    args: PhantomData<Args>,
}

impl<Fn, Fut, Args> FnOnceTask<Fn, Fut, Args> {
    fn new(function: Fn) -> Self {
        FnOnceTask {
            function,
            fut: Default::default(),
            args: Default::default(),
        }
    }
}

impl<'a, Fn, Out, Fut> Task<'a> for FnOnceTask<Fn, Fut, ()>
where
    Fn: FnOnce() -> Fut,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
{
    type Inputs = ();
    type Output = Out;
    type Future = Fut;
    fn run(self, _: Self::Inputs) -> Self::Future {
        (self.function)()
    }
}

impl<'a, Fn, Out, Fut, I1> Task<'a> for FnOnceTask<Fn, Fut, (I1,)>
where
    Fn: FnOnce(I1) -> Fut,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
    I1: Message,
{
    type Inputs = (I1,);
    type Output = Out;
    type Future = Fut;
    fn run(self, inputs: Self::Inputs) -> Self::Future {
        (self.function)(inputs.0)
    }
}

impl<'a, Fn, Out, Fut, I1, I2> Task<'a> for FnOnceTask<Fn, Fut, (I1, I2)>
where
    Fn: FnOnce(I1, I2) -> Fut,
    Out: Message,
    Fut: Future<Output = Out> + Send + 'a,
    I1: Message,
    I2: Message,
{
    type Inputs = (I1, I2);
    type Output = Out;
    type Future = Fut;
    fn run(self, inputs: Self::Inputs) -> Self::Future {
        (self.function)(inputs.0, inputs.1)
    }
}
