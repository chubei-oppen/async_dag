use async_dag::*;
use futures::executor::block_on;

async fn sum(lhs: i32, rhs: i32) -> i32 {
    lhs + rhs
}

fn add_parent_task(graph: &mut Graph, depth: u8, child: NodeIndex) {
    if depth == 0 {
        graph.add_parent_task(|| async { 1i32 }, child, 0).unwrap();
        graph.add_parent_task(|| async { 1i32 }, child, 1).unwrap();
    } else {
        let lhs = graph.add_parent_task(sum, child, 0).unwrap();
        add_parent_task(graph, depth - 1, lhs);
        let rhs = graph.add_parent_task(sum, child, 1).unwrap();
        add_parent_task(graph, depth - 1, rhs);
    }
}

fn main() {
    let mut graph = Graph::new();
    let root = graph.add_task(sum);
    add_parent_task(&mut graph, 10, root);
    block_on(graph.run());
    let result = graph.get_value::<i32>(root).unwrap();
    println!("Result: {}", result);
}
