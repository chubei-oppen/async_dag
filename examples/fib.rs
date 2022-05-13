use async_dag::*;
use futures::executor::block_on;

const N: usize = 44;

async fn sum(lhs: i32, rhs: i32) -> i32 {
    lhs + rhs
}

fn main() {
    let mut graph = Graph::new();
    let mut first = graph.add_task(|| async { 1 });
    let mut second = graph.add_task(|| async { 1 });
    for _ in 0..N {
        let next = graph.add_child_task(sum, first, 0).unwrap();
        graph.update_dependency(second, next, 1).unwrap();

        first = second;
        second = next;
    }
    block_on(graph.run());
    let result = graph.get_value::<i32>(second).unwrap();
    print!("The {}th fibonacci number is {}", N + 2, result);
}
