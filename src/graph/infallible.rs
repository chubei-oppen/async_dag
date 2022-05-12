use super::Edge;
use super::NodeIndex;
use super::TryGraph;
use crate::any::NamedAny;
use crate::error::ErrorWithTask;
use crate::task::IntoInfallibleTask;
use std::convert::Infallible;

/// A [`TryGraph`] with infallible tasks.
pub type Graph<'a> = TryGraph<'a, Infallible>;

impl<'a> Graph<'a> {
    /// Adds an infallible task. See [`TryGraph::add_try_task`].
    pub fn add_task<Args, Ok, T: IntoInfallibleTask<'a, Args, Ok>>(
        &mut self,
        task: T,
    ) -> NodeIndex {
        self.add_task_impl(task.into_task())
    }

    /// Adds a infallible task and set dependency. See [`TryGraph::add_dependent_try_task`].
    pub fn add_dependent_task<Args, Ok: NamedAny, T: IntoInfallibleTask<'a, Args, Ok>>(
        &mut self,
        task: T,
        child: NodeIndex,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T::Task>> {
        self.add_dependent_task_impl::<Ok, _>(task.into_task(), child, index)
    }

    /// Infallible version of [`TryGraph::run`].
    pub async fn run(&mut self) {
        self.try_run().await.unwrap();
    }
}
