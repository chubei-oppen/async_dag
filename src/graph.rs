pub mod error;
mod runner;

use crate::any::type_info;
use crate::any::DynAny;
use crate::any::IntoAny;
use crate::any::TypeInfo;
use crate::curry::CurriedTask;
use crate::curry::Curry;
use crate::task::IntoTryTask;
use crate::task::TryTask;
use crate::tuple::Tuple;
use crate::tuple::TupleIndex;
use daggy::EdgeIndex;
use error::Error;
use error::ErrorWithTask;
use runner::Runner;
use std::any::type_name;
use std::any::Any;
use std::collections::HashMap;

/// A [`Box`]ed [`Curry`].
type DynCurry<'a, Err> = Box<dyn Curry<'a, Err> + 'a>;

impl<'a, Err> std::fmt::Debug for DynCurry<'a, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Curry<{}>", type_name::<Err>()))
            .finish_non_exhaustive()
    }
}

/// Node type.
///
/// A node is either a [`Curry`], running (with a certain output type),
/// or the [`Curry`]'s awaited successful calling output.
#[derive(Debug)]
pub enum Node<'a, Err> {
    /// A [`Curry`].
    Curry(DynCurry<'a, Err>),
    /// A running node.
    ///
    /// The [`Curry`] is called and result future is stored elsewhere and perhaps running.
    Running(TypeInfo),
    /// A successful output from a completed [`TryTask`](crate::task::TryTask).
    Value {
        /// The output value.
        value: DynAny,
        /// The output type.
        type_info: TypeInfo,
    },
}

/// Node identifier.
pub type NodeIndex = daggy::NodeIndex;

/// Edge type.
///
/// An edge connects parent node's task's output to child node's task's input.
/// Its value is the input index.
pub type Edge = TupleIndex;

/// An async task DAG.
#[derive(Debug, Default)]
pub struct TryGraph<'a, Err: 'a> {
    dag: daggy::Dag<Node<'a, Err>, Edge>,
    dependencies: HashMap<(NodeIndex, Edge), EdgeIndex>,
}

impl<'a, Err: 'a> TryGraph<'a, Err> {
    /// Creates an empty [`TryGraph`].
    pub fn new() -> Self {
        Self {
            dag: Default::default(),
            dependencies: Default::default(),
        }
    }

    /// Converts `self` into an iterator of [`Node`]s.
    ///
    /// Client can use this method and previous returned [`NodeIndex`]s to retrive the graph running result.
    pub fn into_nodes(self) -> impl Iterator<Item = Node<'a, Err>> {
        self.dag
            .into_graph()
            .into_nodes_edges()
            .0
            .into_iter()
            .map(|node| node.weight)
    }

    /// Gets the output value of `node`.
    ///
    /// Returns [`None`] if the `node`'s task hasn't done running or the type does not match.
    ///
    /// **Panics** if `node` does not exist within the graph.
    pub fn get_value<T: 'static>(&self, node: NodeIndex) -> Option<T> {
        match self.dag.node_weight(node).unwrap() {
            Node::Value { value, .. } => {
                let value = value.clone().into_any();
                Box::<dyn Any + 'static>::downcast(value)
                    .ok()
                    .map(|value| *value)
            }
            _ => None,
        }
    }

    /// Adds a task without specifying its dependencies.
    ///
    /// Returns the [`NodeIndex`] representing this task.
    ///
    /// **Panics** if the graph is at the maximum number of nodes for its index type.
    pub fn add_try_task<Args, Ok, T: IntoTryTask<'a, Args, Ok, Err>>(
        &mut self,
        task: T,
    ) -> NodeIndex {
        self.add_task_impl(task.into_task())
    }

    fn add_task_impl<T: TryTask<'a, Err = Err> + 'a>(&mut self, task: T) -> NodeIndex {
        self.dag.add_node(Self::make_node(task))
    }

    /// Adds a task and set it as `child`'s dependency at `index`.
    ///
    /// Returns the [`NodeIndex`] representing the added task.
    ///
    /// If child already has a dependency at `index`, it will be removed. But the depended node won't.
    ///
    /// This is more efficient than [`TryGraph::add_task`] then [`TryGraph::update_dependency`].
    ///
    /// **Panics** if the graph is at the maximum number of nodes for its index type.
    ///
    /// **Panics** if `child` does not exist within the graph.
    pub fn add_parent_try_task<Args, Ok: IntoAny, T: IntoTryTask<'a, Args, Ok, Err>>(
        &mut self,
        task: T,
        child: NodeIndex,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T::Task>> {
        self.add_parent_task_impl::<Ok, _>(task.into_task(), child, index)
    }

    fn add_parent_task_impl<Ok: 'static, T: TryTask<'a, Err = Err> + 'a>(
        &mut self,
        task: T,
        child: NodeIndex,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T>> {
        if let Err(error) = self.type_check(child, index, type_info::<Ok>()) {
            return Err(ErrorWithTask { error, task });
        }
        #[allow(unused_results)]
        {
            self.remove_dependency(child, index);
        }
        let (edge, node) = self.dag.add_parent(child, index, Self::make_node(task));
        assert!(self.dependencies.insert((child, index), edge).is_none());
        Ok(node)
    }

    /// Adds a task and set it's dependency at `index` as `parent`.
    ///
    /// Returns the [`NodeIndex`] representing the added task.
    ///
    /// This is more efficient than [`TryGraph::add_task`] then [`TryGraph::update_dependency`].
    ///
    /// **Panics** if the graph is at the maximum number of nodes for its index type.
    ///
    /// **Panics** if `parent` does not exist within the graph.
    pub fn add_child_try_task<Args, Ok: IntoAny, T: IntoTryTask<'a, Args, Ok, Err>>(
        &mut self,
        parent: NodeIndex,
        task: T,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T::Task>> {
        self.add_child_task_impl::<Ok, _>(parent, task.into_task(), index)
    }

    fn add_child_task_impl<Ok: 'static, T: TryTask<'a, Err = Err> + 'a>(
        &mut self,
        parent: NodeIndex,
        task: T,
        index: Edge,
    ) -> Result<NodeIndex, ErrorWithTask<T>> {
        let input_type_info = match T::Inputs::type_info(index) {
            Some(type_info) => type_info,
            None => {
                return Err(ErrorWithTask {
                    error: Error::OutOfRange(T::Inputs::LEN),
                    task,
                })
            }
        };
        let output_type_info = self.output_type_info(parent);
        if let Err(error) = check_type_equality(input_type_info, output_type_info) {
            return Err(ErrorWithTask { error, task });
        }
        let (edge, node) = self.dag.add_child(parent, index, Self::make_node(task));
        assert!(self.dependencies.insert((node, index), edge).is_none());
        Ok(node)
    }

    /// Sets `parent` as `child`'s dependency at `index`.
    ///
    /// If child already has a dependency at `index`, it will be removed. But the depended node won't.
    ///
    /// **Panics** if either `parent` or `child` does not exist within the graph.
    ///
    /// **Panics** if the graph is at the maximum number of edges for its index type.
    pub fn update_dependency(
        &mut self,
        parent: NodeIndex,
        child: NodeIndex,
        index: Edge,
    ) -> Result<(), Error> {
        self.type_check(child, index, self.output_type_info(parent))?;
        #[allow(unused_results)]
        {
            self.remove_dependency(child, index);
        }
        let edge = self
            .dag
            .add_edge(parent, child, index)
            .map_err(|_| Error::WouldCycle)?;
        assert!(self.dependencies.insert((child, index), edge).is_none());
        Ok(())
    }

    /// Remove `child`'s dependency at `index` if it has one.
    ///
    /// Returns `true` if `child` has a dependency at `index` before removing.
    pub fn remove_dependency(&mut self, child: NodeIndex, index: Edge) -> bool {
        let edge = self.dependencies.remove(&(child, index));
        if let Some(edge) = edge {
            assert!(self.dag.remove_edge(edge).is_some());
            true
        } else {
            false
        }
    }

    /// Progresses the whole task graph as much as possible, but aborts on first error.
    ///
    /// If the returned future is dropped before completion, or an error occurs, some tasks will be cancelled and forever lost.
    /// Corresponding [`Node`] will be set to [`Node::Running`].
    pub async fn try_run(&mut self) -> Result<(), Err> {
        let mut runner = Runner::new(&mut self.dag);
        runner.run().await
    }

    fn type_check(
        &self,
        child: NodeIndex,
        index: Edge,
        output_type_info: TypeInfo,
    ) -> Result<(), Error> {
        let node = self.dag.node_weight(child).unwrap();
        let curry = match node {
            Node::Curry(curry) => curry,
            _ => return Err(Error::HasStarted(child)),
        };
        let input_type_info = curry
            .input_type_info(index)
            .ok_or_else(|| Error::OutOfRange(curry.num_inputs()))?;
        check_type_equality(input_type_info, output_type_info)?;
        Ok(())
    }

    fn make_node<T: TryTask<'a, Err = Err> + 'a>(task: T) -> Node<'a, Err> {
        let curry = CurriedTask::new(task);
        Node::Curry(Box::new(curry))
    }

    fn output_type_info(&self, index: NodeIndex) -> TypeInfo {
        let node = self.dag.node_weight(index).unwrap();
        match node {
            Node::Curry(curry) => curry.output_type_info(),
            Node::Running(type_info) => *type_info,
            Node::Value { type_info, .. } => *type_info,
        }
    }
}

fn check_type_equality(input: TypeInfo, output: TypeInfo) -> Result<(), Error> {
    if input != output {
        Err(Error::TypeMismatch { input, output })
    } else {
        Ok(())
    }
}

mod infallible;

pub use infallible::*;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::any::TypeId;

    #[test]
    fn test_diamond_shape_graph() {
        let mut graph = Graph::new();

        let root = graph.add_task(|lhs: i32, rhs: i32| async move { lhs + rhs });
        let lhs = graph
            .add_parent_task(|v: i32| async move { v }, root, 0)
            .unwrap();
        let rhs = graph
            .add_parent_task(|v: i32| async move { v }, root, 1)
            .unwrap();
        let input = graph.add_parent_task(|| async move { 1 }, lhs, 0).unwrap();
        graph.update_dependency(input, rhs, 0).unwrap();

        block_on(graph.run());

        let result = graph.get_value::<i32>(root).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_client_error() {
        let mut graph = TryGraph::new();
        let _ = graph.add_try_task::<_, (), _>(|| async { Err(()) });
        block_on(graph.try_run()).unwrap_err();
    }

    #[test]
    fn test_has_started_check() {
        let mut graph = Graph::new();
        let root = graph.add_task(|_: ()| async { () });
        let parent = graph.add_parent_task(|| async { () }, root, 0).unwrap();
        block_on(graph.run());
        let error = graph.update_dependency(parent, root, 0).unwrap_err();
        let index = match error {
            Error::HasStarted(index) => index,
            _ => panic!("Expecting has started error"),
        };
        assert_eq!(index, root);
    }

    #[test]
    fn test_type_check() {
        let mut graph = Graph::new();
        let root = graph.add_task(|_: ()| async { () });

        let error = graph.type_check(root, 1, type_info::<()>()).unwrap_err();
        let len = match error {
            Error::OutOfRange(len) => len,
            _ => panic!("Expecting out of range error"),
        };
        assert_eq!(len, 1);

        let error = graph.type_check(root, 0, type_info::<i32>()).unwrap_err();
        let (input, output) = match error {
            Error::TypeMismatch { input, output } => (input, output),
            _ => panic!("Expecting type mismatch error"),
        };
        assert_eq!(input.id(), TypeId::of::<()>());
        assert_eq!(output.id(), TypeId::of::<i32>());
        // Name is not guaranteed, but these asserts should be ok...
        assert!(input.name().contains("()"));
        assert!(output.name().contains("i32"));
    }

    #[test]
    fn test_cycle_check() {
        let mut graph = Graph::new();
        let root = graph.add_task(|_: ()| async { () });
        let parent = graph
            .add_parent_task(|_: ()| async { () }, root, 0)
            .unwrap();
        let error = graph.update_dependency(root, parent, 0).unwrap_err();
        match error {
            Error::WouldCycle => (),
            _ => panic!("Expecting would cycle error"),
        }
    }

    #[test]
    fn test_remove_dependency() {
        let mut graph = Graph::new();
        let root = graph.add_task(|_: ()| async { () });
        assert!(!graph.remove_dependency(root, 0));
        let _ = graph.add_parent_task(|| async { () }, root, 0).unwrap();
        assert!(graph.remove_dependency(root, 0));
    }

    #[test]
    fn test_update_dependency() {
        let mut graph = Graph::new();
        let root = graph.add_task(|_: ()| async { () });
        let parent = graph.add_parent_task(|| async { () }, root, 0).unwrap();
        graph.update_dependency(parent, root, 0).unwrap();
        graph.update_dependency(parent, root, 0).unwrap();
    }
}
