use daggy::petgraph::Direction;

mod any;
mod curry;
mod node;
mod runner;
mod task;
pub mod tuple;

pub use any::DynAny;
pub use any::NamedAny;
use curry::CurriedTask;
pub use curry::Curry;
pub use runner::GraphError;
pub use runner::IncorrectDependency;
use runner::Runner;
pub use task::IntoTryTask;
pub use task::TryTask;
use tuple::TupleIndex;

/// Type eraser of [`Curry`].
pub type DynCurry<'a, Err> = Box<dyn Curry<'a, Err> + 'a>;

/// Node type.
///
/// A node is either a [`Curry`], running, or the [`Curry`]'s awaited calling result.
pub enum Node<'a, Err> {
    /// A [`Curry`].
    Curry(DynCurry<'a, Err>),
    /// A running node.
    ///
    /// The [`Curry`] is called and result future is stored elsewhere and perhaps running.
    Running,
    /// A successful output from a completed [`TryTask`].
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
pub struct Graph<'a, Err: 'a>(daggy::Dag<Node<'a, Err>, Edge>);

/// An error returned by the [Graph::add_dependency] method in the case that adding
/// an dependency would have caused the graph to cycle.
pub type WouldCycle = daggy::WouldCycle<Edge>;

impl<'a, Err: 'a> Graph<'a, Err> {
    /// Creates an empty [`Graph`].
    pub fn new() -> Self {
        Self(daggy::Dag::new())
    }

    /// Converts `self` into an iterator of [`Node`]s.
    ///
    /// Client should use this method and previous returned [`NodeIndex`]s to retrive the graph running result.
    pub fn into_nodes(self) -> impl Iterator<Item = Node<'a, Err>> {
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
    pub fn add_task<Args, Ok, T: IntoTryTask<'a, Args, Ok, Err>>(&mut self, task: T) -> NodeIndex {
        self.0.add_node(Self::make_node(task))
    }

    /// Adds a task and set it as `child`'s dependency.
    ///
    /// Returns the [`NodeIndex`] representing the added task.
    ///
    /// This is more efficient than [`Graph::add_task`] then [`Graph::add_dependency`].
    pub fn add_dependent_task<Args, Ok, T: IntoTryTask<'a, Args, Ok, Err>>(
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
    /// Client can check the error, modify the [`Graph`] accordingly (TODO), and try running again.
    ///
    /// If the returned future is dropped before completion, some tasks will be cancelled and forever lost.
    /// Corresponding [`Node`] will be set to [`Node::Running`].
    pub async fn run(&mut self) -> Result<(), GraphError<Err>> {
        let mut runner = Runner::new(&mut self.0);
        runner.run().await
    }

    fn make_node<Args, Ok, T: IntoTryTask<'a, Args, Ok, Err>>(task: T) -> Node<'a, Err> {
        let task = task.into_task();
        let curry = CurriedTask::new(task);
        Node::Curry(Box::new(curry))
    }
}

impl<'a, Err> Default for Graph<'a, Err> {
    fn default() -> Self {
        Self::new()
    }
}
