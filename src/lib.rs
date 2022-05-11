use daggy::petgraph::Direction;

mod curry;
mod node;
mod runner;
mod task;
pub mod tuple;

use curry::CurriedTask;
pub use curry::Curry;
pub use runner::IncorrectDependency;
use runner::Runner;
pub use task::IntoTask;
pub use task::Message;
pub use task::Task;
use tuple::DynAny;
use tuple::TupleIndex;

/// Type eraser of [`Curry`].
pub type DynCurry<'a> = Box<dyn Curry<'a> + 'a>;

/// Node type.
///
/// A node is either a [`Curry`], running, or the [`Curry`]'s awaited calling result.
pub enum Node<'a> {
    /// A [`Curry`].
    Curry(DynCurry<'a>),
    /// A running node.
    ///
    /// The future is stored elsewhere. Client should not see this state.
    Running,
    /// A result from a completed [`Task`].
    Value(DynAny),
}

/// Node identifier.
pub type NodeIndex = daggy::NodeIndex;

/// Edge type.
///
/// An edge connects parent node's output to child node's input.
/// Its value is the input index.
pub type Edge = TupleIndex;

/// An async task DAG.
pub struct Graph<'a>(daggy::Dag<Node<'a>, Edge>);

/// An error returned by the [Graph::add_dependency] method in the case that adding
/// an dependency would have caused the graph to cycle.
pub type WouldCycle = daggy::WouldCycle<Edge>;

impl<'a> Graph<'a> {
    /// Creates an empty [`Graph`].
    pub fn new() -> Self {
        Self(daggy::Dag::new())
    }

    /// Converts `self` into an iterator of [`Node`]s.
    ///
    /// Client should use this method and previous returned [`NodeIndex`]s to retrive the graph running result.
    pub fn into_nodes(self) -> impl Iterator<Item = Node<'a>> {
        self.0
            .into_graph()
            .into_nodes_edges()
            .0
            .into_iter()
            .map(|node| node.weight)
    }

    /// Adds a task without specifying its dependencies.
    ///
    /// Returns the [`NodeIndex`] representing this task.
    pub fn add_task<Args, Out, T: IntoTask<'a, Args, Out>>(&mut self, task: T) -> NodeIndex {
        self.0.add_node(Self::make_node(task))
    }

    /// Adds a task and set it as `child`'s dependency.
    ///
    /// Returns the [`NodeIndex`] representing the added task.
    ///
    /// This is more efficient than [`Graph::add_task`] then [`Graph::add_dependency`].
    pub fn add_dependent_task<Args, Out, T: IntoTask<'a, Args, Out>>(
        &mut self,
        child: NodeIndex,
        task: T,
    ) -> NodeIndex {
        let current_count = self
            .0
            .graph()
            .edges_directed(child, Direction::Incoming)
            .count() as u8;
        self.0
            .add_parent(child, current_count, Self::make_node(task))
            .1
    }

    /// Sets `parent` as `child`'s dependency.
    pub fn add_dependency(
        &mut self,
        child: NodeIndex,
        parent: NodeIndex,
    ) -> Result<(), WouldCycle> {
        let current_count = self
            .0
            .graph()
            .edges_directed(child, Direction::Incoming)
            .count() as u8;
        self.0.add_edge(parent, child, current_count).map(|_| ())
    }

    /// Progresses the whole task graph as much as possible.
    ///
    /// Returns the result [`Graph`] and any error that happens during running.
    ///
    /// Client can check the error, modify the [`Graph`] accordingly (TODO), and try running again.
    pub async fn run(self) -> (Graph<'a>, Option<IncorrectDependency>) {
        let graph = self.0;
        let runner = Runner::new(graph);
        let (graph, error) = runner.run().await;
        (Self(graph), error)
    }

    fn make_node<Args, Out, T: IntoTask<'a, Args, Out>>(task: T) -> Node<'a> {
        let task = task.into_task();
        let curry = CurriedTask::new(task);
        Node::Curry(Box::new(curry))
    }
}

impl<'a> Default for Graph<'a> {
    fn default() -> Self {
        Self::new()
    }
}
