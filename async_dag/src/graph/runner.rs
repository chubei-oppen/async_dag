use crate::any::DynAny;
use crate::any::TypeInfo;
use crate::curry::TaskFuture;
use crate::graph::Edge;
use crate::graph::Node;
use crate::graph::NodeIndex;
use daggy::petgraph::visit::EdgeRef;
use daggy::petgraph::visit::IntoEdgesDirected;
use daggy::petgraph::Direction;
use daggy::Dag;
use futures::future::select_all;
use futures::FutureExt;
use std::future::Future;
use std::mem::swap;
use std::task::Poll;

struct RunningNode<'a, Err> {
    index: NodeIndex,
    future: TaskFuture<'a, Err>,
}

impl<'a, Err> Future for RunningNode<'a, Err> {
    type Output = (NodeIndex, Result<DynAny, Err>);

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
fn call_node<'a, Err>(node: &mut Node<'a, Err>) -> Option<TaskFuture<'a, Err>> {
    // Make a placeholder and swap `node` out.
    let mut owned_node = Node::Running(TypeInfo::of::<()>());
    swap(node, &mut owned_node);

    if let Node::Curry(curry) = owned_node {
        if curry.ready() {
            *node = Node::Running(curry.output_type_info());
            Some(curry.call().unwrap())
        } else {
            *node = Node::Curry(curry);
            None
        }
    } else {
        *node = owned_node;
        None
    }
}

/// The async DAG driver algorithm.
pub struct Runner<'task, 'graph, Err> {
    // We only modify node weights inside `node_graph`, don't change its structure.
    node_graph: &'graph mut Dag<Node<'task, Err>, Edge>,
    // `edge_graph` has the same structure as `node_graph`,
    // so we can access connection information and modify node weights simutaneously.
    edge_graph: Dag<(), Edge>,
    running: Vec<RunningNode<'task, Err>>,
}

impl<'task, 'graph, Err> Runner<'task, 'graph, Err> {
    /// Creates a new runner from a [Graph].
    ///
    /// The `graph` must have been type checked.
    /// If dropped before running completes, some tasks will be cancelled and forever lost.
    pub fn new(graph: &'graph mut Dag<Node<'task, Err>, Edge>) -> Self {
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
    /// If the returned future is dropped before completion or client error happens,
    /// some tasks will be cancelled and forever lost.
    pub async fn run(&mut self) -> Result<(), Err> {
        while !self.running.is_empty() {
            self.step().await?;
        }
        Ok(())
    }

    /// Polls until one running node is completed.
    ///
    /// Curries dependent nodes and returns early on error.
    async fn step(&mut self) -> Result<(), Err> {
        // Swap out `self.running` for `select_all`.
        let mut running = vec![];
        swap(&mut self.running, &mut running);

        // If client error happens, return early and drop running futures.
        let ((node_index, result), _, running) = select_all(running).await;
        let output = result?;

        // Assign back to `self.running`.
        self.running = running;

        // Traverse outgoing edges of completed node.
        for edge in self
            .edge_graph
            .edges_directed(node_index, Direction::Outgoing)
        {
            let child_index = edge.target();
            let child_node = self.node_graph.node_weight_mut(child_index).unwrap();

            if let Node::Curry(curry) = child_node {
                let input_index = *edge.weight();
                curry.curry(input_index, output.clone()).unwrap();
            }

            if let Some(future) = call_node(child_node) {
                self.running.push(RunningNode {
                    index: child_index,
                    future,
                });
            }
        }

        let node = self.node_graph.node_weight_mut(node_index).unwrap();
        // It must be `Running`.
        let type_info = match node {
            Node::Running(type_info) => *type_info,
            _ => panic!("Expecting running state"),
        };
        *self.node_graph.node_weight_mut(node_index).unwrap() = Node::Value {
            value: output,
            type_info,
        };

        Ok(())
    }
}
