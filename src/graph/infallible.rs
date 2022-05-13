use super::Edge;
use super::NodeIndex;
use super::TryGraph;
use crate::any::IntoAny;
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

    /// Adds an infallible task and set it as `child`'s dependency at `index`.
    ///
    /// See [`TryGraph::add_parent_try_task`].
    pub fn add_parent_task<Args, Ok: IntoAny, T: IntoInfallibleTask<'a, Args, Ok>>(
        &mut self,
        task: T,
        child: NodeIndex,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T::Task>> {
        self.add_parent_task_impl::<Ok, _>(task.into_task(), child, index)
    }

    /// Adds an infallible task and set it's dependency at `index` to `parent`.
    ///
    /// See [`TryGraph::add_child_try_task`].
    pub fn add_child_task<Args, Ok: IntoAny, T: IntoInfallibleTask<'a, Args, Ok>>(
        &mut self,
        task: T,
        parent: NodeIndex,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T::Task>> {
        self.add_child_task_impl::<Ok, _>(task.into_task(), parent, index)
    }

    /// Infallible version of [`TryGraph::run`].
    pub async fn run(&mut self) {
        self.try_run().await.unwrap();
    }
}
