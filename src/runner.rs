use super::Edge;
use super::Node;
use super::NodeIndex;
use crate::curry::DynMessage;
use crate::curry::TaskFuture;
pub use crate::tuple::InsertErrorKind;
use daggy::petgraph::visit::EdgeRef;
use daggy::petgraph::visit::IntoEdgesDirected;
use daggy::petgraph::Direction;
use daggy::Dag;
use futures::future::select_all;
use futures::FutureExt;
use std::future::Future;
use std::mem::swap;
use std::task::Poll;

#[derive(Debug)]
/// One of the dependency setup is incorrect.
pub struct IncorrectDependency {
    /// The error kind.
    pub kind: InsertErrorKind,
    /// The depended node index.
    pub parent: NodeIndex,
    /// The dependent node index.
    pub child: NodeIndex,
}

impl std::fmt::Display for IncorrectDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncorrectDependency")
            .field("kind", &self.kind)
            .field("parent", &self.parent)
            .field("child", &self.child)
            .finish()
    }
}

impl std::error::Error for IncorrectDependency {}

struct RunningNode<'a> {
    index: NodeIndex,
    future: TaskFuture<'a>,
}

impl<'a> Future for RunningNode<'a> {
    type Output = (NodeIndex, DynMessage);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.future.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((self.index, output)),
        }
    }
}

// Puts `node` to running if it contains a ready [Curry], doesn't change it otherwise.
fn call_node<'a>(node: &mut Node<'a>) -> Option<TaskFuture<'a>> {
    let mut temp = Node::Running;
    swap(node, &mut temp);
    if let Node::Curry(curry) = temp {
        if curry.ready() {
            Some(curry.call().unwrap())
        } else {
            *node = Node::Curry(curry);
            None
        }
    } else {
        *node = temp;
        None
    }
}

/// The async DAG driver algorithm.
pub struct Runner<'task, 'graph> {
    // We only modify node weights inside `node_graph`, don't change its structure.
    node_graph: &'graph mut Dag<Node<'task>, Edge>,
    // `edge_graph` has the same structure as `node_graph`,
    // so we can access connection information and modify node weights simutaneously.
    edge_graph: Dag<(), Edge>,
    running: Vec<RunningNode<'task>>,
}

impl<'task, 'graph> Runner<'task, 'graph> {
    /// Creates a new runner from a [Graph].
    ///
    /// If dropped before running completes, some tasks will be cancelled and forever lost.
    pub fn new(graph: &'graph mut Dag<Node<'task>, Edge>) -> Self {
        let mut running = vec![];

        for index in 0..graph.node_count() {
            let index = NodeIndex::new(index);
            let node = graph.node_weight_mut(index).unwrap();
            if let Some(future) = call_node(node) {
                running.push(RunningNode { index, future });
            }
        }

        let edge_graph = graph.map(|_, _| (), |_, edge| *edge);

        Self {
            node_graph: graph,
            edge_graph,
            running,
        }
    }

    /// Runs the algorithm.
    ///
    /// If the returned future is dropped before completion, some tasks will be cancelled and forever lost.
    pub async fn run(&mut self) -> Result<(), IncorrectDependency> {
        while !self.running.is_empty() {
            self.step().await?;
        }
        Ok(())
    }

    /// Polls until one running node is completed.
    ///
    /// Curries dependent nodes and returns early on error.
    async fn step(&mut self) -> Result<(), IncorrectDependency> {
        let mut running = vec![];
        swap(&mut self.running, &mut running);
        let ((node_index, output), _, running) = select_all(running).await;
        self.running = running;

        for edge in self
            .edge_graph
            .edges_directed(node_index, Direction::Outgoing)
        {
            let child_index = edge.target();
            let child_node = self.node_graph.node_weight_mut(child_index).unwrap();

            if let Node::Curry(curry) = child_node {
                if let Err(error) = curry.curry(*edge.weight(), output.clone_any()) {
                    // Save output and return error.
                    *self.node_graph.node_weight_mut(node_index).unwrap() =
                        Node::Value(output.into_any());
                    let error = IncorrectDependency {
                        kind: error.kind,
                        parent: edge.source(),
                        child: child_index,
                    };
                    return Err(error);
                }
            }

            if let Some(future) = call_node(child_node) {
                self.running.push(RunningNode {
                    index: child_index,
                    future,
                });
            }
        }

        *self.node_graph.node_weight_mut(node_index).unwrap() = Node::Value(output.into_any());

        Ok(())
    }
}
