use super::TryTask;
use crate::any::IntoAny;
use futures::future::FutureExt;
use futures::future::Map;
use seq_macro::seq;
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

macro_rules! task_impl {
    ($N:literal) => {
        seq!(i in 0..$N {
            impl<'a, Fn, Ok, Fut, #(I~i,)*> IntoInfallibleTask<'a, (#(I~i,)*), Ok> for Fn
            where
                Fn: FnOnce(#(I~i,)*) -> Fut + 'a,
                Ok: IntoAny,
                Fut: Future<Output = Ok> + Send + 'a,
                #(
                    I~i: IntoAny,
                )*
            {
                type Task = InfallibleFnOnceTask<Fn, Ok, Fut, (#(I~i,)*)>;

                fn into_task(self) -> Self::Task {
                    InfallibleFnOnceTask::new(self)
                }
            }

            impl<'a, Fn, Ok, Fut, #(I~i,)*> TryTask<'a> for InfallibleFnOnceTask<Fn, Ok, Fut, (#(I~i,)*)>
            where
                Fn: FnOnce(#(I~i,)*) -> Fut,
                Ok: IntoAny,
                Fut: Future<Output = Ok> + Send + 'a,
                #(
                    I~i: IntoAny,
                )*
            {
                type Inputs = (#(I~i,)*);
                type Ok = Ok;
                type Err = Infallible;
                type Future = Map<Fut, fn(Ok) -> Result<Ok, Infallible>>;
                fn run(self, (#(v~i,)*): Self::Inputs) -> Self::Future {
                    (self.function)(#(v~i,)*).map(Ok)
                }
            }
        });
    };
}

seq!(N in 0..=12 {
    task_impl!(N);
});
