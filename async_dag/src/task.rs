use crate::any::IntoAny;
use crate::tuple::Tuple;
use seq_macro::seq;
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

macro_rules! task_impl {
    ($N:literal) => {
        seq!(i in 0..$N {
            impl<'a, Fn, Ok, Err, Fut, #(I~i,)*> IntoTryTask<'a, (#(I~i,)*), Ok, Err> for Fn
            where
                Fn: FnOnce(#(I~i,)*) -> Fut + 'a,
                Ok: IntoAny,
                Err: 'a,
                Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
                #(
                    I~i: IntoAny,
                )*
            {
                type Task = FnOnceTask<Fn, Ok, Err, Fut, (#(I~i,)*)>;

                fn into_task(self) -> Self::Task {
                    FnOnceTask::new(self)
                }
            }

            impl<'a, Fn, Ok, Err, Fut, #(I~i,)*> TryTask<'a> for FnOnceTask<Fn, Ok, Err, Fut, (#(I~i,)*)>
            where
                Fn: FnOnce(#(I~i,)*) -> Fut,
                Ok: IntoAny,
                Err: 'a,
                Fut: Future<Output = Result<Ok, Err>> + Send + 'a,
                #(
                    I~i: IntoAny,
                )*
            {
                type Inputs = (#(I~i,)*);
                type Ok = Ok;
                type Err = Err;
                type Future = Fut;
                fn run(self, (#(v~i,)*): Self::Inputs) -> Self::Future {
                    (self.function)(#(v~i,)*)
                }
            }
        });
    };
}

seq!(N in 0..=12 {
    task_impl!(N);
});

mod infallible;

pub use infallible::*;
