use crate::any::NamedAny;
use crate::tuple::Tuple;
use std::future::Future;
use std::marker::PhantomData;

/// An async task.
pub trait TryTask<'a> {
    /// Tuple of inputs.
    type Inputs: Tuple;

    /// Successful output.
    type Ok: NamedAny;

    /// Error output.
    type Err: 'a;

    /// Output future.
    type Future: Future<Output = Result<Self::Ok, Self::Err>> + Send + 'a;

    /// Runs the task and gets a future.
    fn run(self, inputs: Self::Inputs) -> Self::Future;
}

/// Conversion into a [`TryTask`].
pub trait IntoTryTask<'a, Args, Ok, Err> {
    /// The [`TryTask`] type.
    type Task: TryTask<'a, Ok = Ok, Err = Err> + 'a;

    /// The conversion.
    fn into_task(self) -> Self::Task;
}

impl<'a, Fn, Ok, Err, Fut> IntoTryTask<'a, (), Ok, Err> for Fn
where
    Fn: FnOnce() -> Fut + 'a,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
{
    type Task = FnOnceTask<Fn, Fut, ()>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Err, Fut, I1> IntoTryTask<'a, (I1,), Ok, Err> for Fn
where
    Fn: FnOnce(I1) -> Fut + 'a,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I1: NamedAny,
{
    type Task = FnOnceTask<Fn, Fut, (I1,)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Err, Fut, I1, I2> IntoTryTask<'a, (I1, I2), Ok, Err> for Fn
where
    Fn: FnOnce(I1, I2) -> Fut + 'a,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I1: NamedAny,
    I2: NamedAny,
{
    type Task = FnOnceTask<Fn, Fut, (I1, I2)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

/// A [`TryTask`] for types that implement [`FnOnce`].
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

impl<'a, Fn, Ok, Err, Fut> TryTask<'a> for FnOnceTask<Fn, Fut, ()>
where
    Fn: FnOnce() -> Fut,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
{
    type Inputs = ();
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, _: Self::Inputs) -> Self::Future {
        (self.function)()
    }
}

impl<'a, Fn, Ok, Err, Fut, I1> TryTask<'a> for FnOnceTask<Fn, Fut, (I1,)>
where
    Fn: FnOnce(I1) -> Fut,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I1: NamedAny,
{
    type Inputs = (I1,);
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, inputs: Self::Inputs) -> Self::Future {
        (self.function)(inputs.0)
    }
}

impl<'a, Fn, Ok, Err, Fut, I1, I2> TryTask<'a> for FnOnceTask<Fn, Fut, (I1, I2)>
where
    Fn: FnOnce(I1, I2) -> Fut,
    Ok: NamedAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I1: NamedAny,
    I2: NamedAny,
{
    type Inputs = (I1, I2);
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, inputs: Self::Inputs) -> Self::Future {
        (self.function)(inputs.0, inputs.1)
    }
}
