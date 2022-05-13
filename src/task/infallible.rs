use super::TryTask;
use crate::any::IntoAny;
use futures::future::FutureExt;
use futures::future::Map;
use std::any::type_name;
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;

/// Conversion to a [`Infallible`] [`TryTask`].
pub trait IntoInfallibleTask<'a, Args, Ok> {
    /// The [`TryTask`] type.
    type Task: TryTask<'a, Ok = Ok, Err = Infallible> + 'a;

    /// The conversion.
    fn into_task(self) -> Self::Task;
}

impl<'a, Fn, Ok, Fut> IntoInfallibleTask<'a, (), Ok> for Fn
where
    Fn: FnOnce() -> Fut + 'a,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
{
    type Task = InfallibleFnOnceTask<Fn, Ok, Fut, ()>;

    fn into_task(self) -> Self::Task {
        InfallibleFnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Fut, I0> IntoInfallibleTask<'a, (I0,), Ok> for Fn
where
    Fn: FnOnce(I0) -> Fut + 'a,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
    I0: IntoAny,
{
    type Task = InfallibleFnOnceTask<Fn, Ok, Fut, (I0,)>;

    fn into_task(self) -> Self::Task {
        InfallibleFnOnceTask::new(self)
    }
}

impl<'a, Fn, Ok, Fut, I0, I1> IntoInfallibleTask<'a, (I0, I1), Ok> for Fn
where
    Fn: FnOnce(I0, I1) -> Fut + 'a,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
{
    type Task = InfallibleFnOnceTask<Fn, Ok, Fut, (I0, I1)>;

    fn into_task(self) -> Self::Task {
        InfallibleFnOnceTask::new(self)
    }
}

/// A [`Infallible`] [`TryTask`] for types that implement [`FnOnce`].
pub struct InfallibleFnOnceTask<Fn, Ok, Fut, Args> {
    function: Fn,
    ok: PhantomData<Ok>,
    fut: PhantomData<Fut>,
    args: PhantomData<Args>,
}

impl<Fn, Ok, Fut, Args> InfallibleFnOnceTask<Fn, Ok, Fut, Args> {
    fn new(function: Fn) -> Self {
        InfallibleFnOnceTask {
            function,
            ok: Default::default(),
            fut: Default::default(),
            args: Default::default(),
        }
    }
}

impl<Fn, Ok, Fut, Args> std::fmt::Debug for InfallibleFnOnceTask<Fn, Ok, Fut, Args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "InfallibleFnOnceTask{} -> impl Future<Output = {}> {{ ... }}",
            type_name::<Args>(),
            type_name::<Ok>(),
        ))
    }
}

impl<'a, Fn, Ok, Fut> TryTask<'a> for InfallibleFnOnceTask<Fn, Ok, Fut, ()>
where
    Fn: FnOnce() -> Fut,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
{
    type Inputs = ();
    type Ok = Ok;
    type Err = Infallible;
    type Future = Map<Fut, fn(Ok) -> Result<Ok, Infallible>>;
    fn run(self, _: Self::Inputs) -> Self::Future {
        (self.function)().map(Ok)
    }
}

impl<'a, Fn, Ok, Fut, I0> TryTask<'a> for InfallibleFnOnceTask<Fn, Ok, Fut, (I0,)>
where
    Fn: FnOnce(I0) -> Fut,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
    I0: IntoAny,
{
    type Inputs = (I0,);
    type Ok = Ok;
    type Err = Infallible;
    type Future = Map<Fut, fn(Ok) -> Result<Ok, Infallible>>;
    fn run(self, (i0,): Self::Inputs) -> Self::Future {
        (self.function)(i0).map(Ok)
    }
}

impl<'a, Fn, Ok, Fut, I0, I1> TryTask<'a> for InfallibleFnOnceTask<Fn, Ok, Fut, (I0, I1)>
where
    Fn: FnOnce(I0, I1) -> Fut,
    Ok: IntoAny,
    Fut: Future<Output = Ok> + Send + 'a,
    I0: IntoAny,
    I1: IntoAny,
{
    type Inputs = (I0, I1);
    type Ok = Ok;
    type Err = Infallible;
    type Future = Map<Fut, fn(Ok) -> Result<Ok, Infallible>>;
    fn run(self, (i0, i1): Self::Inputs) -> Self::Future {
        (self.function)(i0, i1).map(Ok)
    }
}
