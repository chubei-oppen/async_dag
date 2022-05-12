use async_dag::*;
use futures::executor::block_on;
use std::any::Any;

async fn sum(lhs: i32, rhs: i32) -> i32 {
    lhs + rhs
}

fn add_dependent_task(graph: &mut Graph, depth: u8, child: NodeIndex) {
    if depth == 0 {
        graph
            .add_dependent_task(|| async { 1i32 }, child, 0)
            .unwrap();
        graph
            .add_dependent_task(|| async { 1i32 }, child, 1)
            .unwrap();
    } else {
        let lhs = graph.add_dependent_task(sum, child, 0).unwrap();
        add_dependent_task(graph, depth - 1, lhs);
        let rhs = graph.add_dependent_task(sum, child, 1).unwrap();
        add_dependent_task(graph, depth - 1, rhs);
    }
}

fn main() {
    let mut graph = Graph::new();
    let root = graph.add_task(sum);
    add_dependent_task(&mut graph, 10, root);
    block_on(graph.run());
    let result = graph.into_nodes().nth(root.index()).unwrap();
    if let Node::Value(result) = result {
        let result: Box<i32> = Box::<dyn Any + 'static>::downcast(result.into_any()).unwrap();
        println!("Result: {}", *result);
    } else {
        unreachable!();
    }
}
