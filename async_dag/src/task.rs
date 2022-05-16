use crate::any::IntoAny;
use crate::tuple::Tuple;
use std::any::type_name;
use std::future::Future;
use std::marker::PhantomData;

/// An async task.
pub trait TryTask<'a>: std::fmt::Debug {
    /// Tuple of inputs.
    type Inputs: Tuple;

    /// Successful output.
    type Ok: IntoAny;

    /// Error output.
    type Err: 'a;

    /// Output future.
    type Future: Future<Output = Result<Self::Ok, Self::Err>> + Send + 'a;

    /// Runs the task and gets a future.
    fn run(self, inputs: Self::Inputs) -> Self::Future;
}

/// Conversion to a [`TryTask`].
pub trait IntoTryTask<'a, Args, Ok, Err> {
    /// The [`TryTask`] type.
    type Task: TryTask<'a, Ok = Ok, Err = Err> + 'a;

    /// The conversion.
    fn into_task(self) -> Self::Task;
}

impl<'a, Fn, Ok, Err, Fut> IntoTryTask<'a, (), Ok, Err> for Fn
where
    Fn: FnOnce() -> Fut + 'a,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
{
    type Task = FnOnceTask<Fn, Ok, Err, Fut, ()>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Err, Fut, I0> IntoTryTask<'a, (I0,), Ok, Err> for Fn
where
    Fn: FnOnce(I0) -> Fut + 'a,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
{
    type Task = FnOnceTask<Fn, Ok, Err, Fut, (I0,)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Err, Fut, I0, I1> IntoTryTask<'a, (I0, I1), Ok, Err> for Fn
where
    Fn: FnOnce(I0, I1) -> Fut + 'a,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
{
    type Task = FnOnceTask<Fn, Ok, Err, Fut, (I0, I1)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Err, Fut, I0, I1, I2> IntoTryTask<'a, (I0, I1, I2), Ok, Err> for Fn
where
    Fn: FnOnce(I0, I1, I2) -> Fut + 'a,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
    I2: IntoAny,
{
    type Task = FnOnceTask<Fn, Ok, Err, Fut, (I0, I1, I2)>;

    fn into_task(self) -> Self::Task {
        FnOnceTask::new(self)
    }
}

/// A [`TryTask`] for types that implement [`FnOnce`].
pub struct FnOnceTask<Fn, Ok, Err, Fut, Args> {
    function: Fn,
    ok: PhantomData<Ok>,
    err: PhantomData<Err>,
    fut: PhantomData<Fut>,
    args: PhantomData<Args>,
}

impl<Fn, Ok, Err, Fut, Args> FnOnceTask<Fn, Ok, Err, Fut, Args> {
    fn new(function: Fn) -> Self {
        FnOnceTask {
            function,
            ok: Default::default(),
            err: Default::default(),
            fut: Default::default(),
            args: Default::default(),
        }
    }
}

impl<Fn, Ok, Err, Fut, Args> std::fmt::Debug for FnOnceTask<Fn, Ok, Err, Fut, Args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "FnOnceTask{} -> impl Future<Output = Result<{}, {}> {{ ... }}",
            type_name::<Args>(),
            type_name::<Ok>(),
            type_name::<Err>(),
        ))
    }
}

impl<'a, Fn, Ok, Err, Fut> TryTask<'a> for FnOnceTask<Fn, Ok, Err, Fut, ()>
where
    Fn: FnOnce() -> Fut,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
{
    type Inputs = ();
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, (): Self::Inputs) -> Self::Future {
        (self.function)()
    }
}

impl<'a, Fn, Ok, Err, Fut, I0> TryTask<'a> for FnOnceTask<Fn, Ok, Err, Fut, (I0,)>
where
    Fn: FnOnce(I0) -> Fut,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
{
    type Inputs = (I0,);
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, (i0,): Self::Inputs) -> Self::Future {
        (self.function)(i0)
    }
}

impl<'a, Fn, Ok, Err, Fut, I0, I1> TryTask<'a> for FnOnceTask<Fn, Ok, Err, Fut, (I0, I1)>
where
    Fn: FnOnce(I0, I1) -> Fut,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
{
    type Inputs = (I0, I1);
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, (i0, i1): Self::Inputs) -> Self::Future {
        (self.function)(i0, i1)
    }
}

impl<'a, Fn, Ok, Err, Fut, I0, I1, I2> TryTask<'a> for FnOnceTask<Fn, Ok, Err, Fut, (I0, I1, I2)>
where
    Fn: FnOnce(I0, I1, I2) -> Fut,
    Ok: IntoAny,
    Err: 'a,
    Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
    I2: IntoAny,
{
    type Inputs = (I0, I1, I2);
    type Ok = Ok;
    type Err = Err;
    type Future = Fut;
    fn run(self, (i0, i1, i2): Self::Inputs) -> Self::Future {
        (self.function)(i0, i1, i2)
    }
}

mod infallible;

pub use infallible::*;
