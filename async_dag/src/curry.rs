use crate::any::DynAny;
use crate::any::IntoAny;
use crate::any::TypeInfo;
use crate::task::TryTask;
use crate::tuple::InsertResult;
use crate::tuple::TakeError;
use crate::tuple::Tuple;
use crate::tuple::TupleIndex;
use crate::tuple::TupleOption;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::TryFutureExt;

pub type TaskFuture<'a, Err> = BoxFuture<'a, Result<DynAny, Err>>;

/// [`Curry`] describes the process of currying and finally calling.
pub trait Curry<'a, Err> {
    /// The number of inputs of the original task.
    fn num_inputs(&self) -> TupleIndex;

    /// Returns the [`TypeInfo`] of the input at `index`, [`None`] if `index` is out of range.
    fn input_type_info(&self, index: TupleIndex) -> Option<TypeInfo>;

    /// Returns the [`TypeInfo`] of the successful output.
    fn output_type_info(&self) -> TypeInfo;

    /// Returns `true` if the inner task's inputs has been populated and becomes ready for running.
    fn ready(&self) -> bool;

    /// Inserts a input to the inner task, i.e. currying.
    ///
    /// `self` is unchanged on error.
    fn curry(&mut self, index: TupleIndex, value: DynAny) -> InsertResult;

    /// Consumes the inner task and inputs and returns a future of the output value.
    fn call(self: Box<Self>) -> Result<TaskFuture<'a, Err>, TakeError>;
}

/// [`CurriedTask`] holds a task and its inputs and tracks if all inputs are ready.
pub struct CurriedTask<'a, Err, T: TryTask<'a, Err = Err>> {
    task: T,
    inputs: <T::Inputs as Tuple>::Option,
}

impl<'a, Err, T: TryTask<'a, Err = Err>> CurriedTask<'a, Err, T> {
    /// Creates a [CurriedTask] from a task and no inputs.
    pub fn new(task: T) -> Self {
        CurriedTask {
            task,
            inputs: Default::default(),
        }
    }
}

fn make_any<T: IntoAny>(t: T) -> DynAny {
    Box::new(t)
}

impl<'a, Err, T: TryTask<'a, Err = Err>> Curry<'a, Err> for CurriedTask<'a, Err, T> {
    fn num_inputs(&self) -> TupleIndex {
        T::Inputs::LEN
    }

    fn input_type_info(&self, index: TupleIndex) -> Option<TypeInfo> {
        T::Inputs::type_info(index)
    }

    fn output_type_info(&self) -> TypeInfo {
        TypeInfo::of::<T::Ok>()
    }

    fn ready(&self) -> bool {
        self.inputs.first_none().is_none()
    }

    fn curry(&mut self, index: TupleIndex, value: DynAny) -> InsertResult {
        self.inputs.insert(index, value)
    }

    fn call(self: Box<Self>) -> Result<TaskFuture<'a, Err>, TakeError> {
        let CurriedTask { task, mut inputs } = *self;
        let inputs = inputs.take()?;
        let future = task.run(inputs);
        let future = future.map_ok(make_any);
        Ok(future.boxed())
    }
}
