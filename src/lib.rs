//! `async_dag` is an async task scheduling utility.
//!
//! When async tasks and their dependencies can be described by a [DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph),
//! this crate ensures the tasks are run at maximum posiible parallelism.
//!
//! # Example
//!
//! Say there are several tasks which either produces an `i32` or sums two `i32`s,
//! and they have dependency relationship described by following graph,
//!
//! ```text
//!       7
//!      / \
//!     3   \
//!    / \   \
//!   1   2   4
//! ```
//!
//! which means there are three tasks producing value `1`, `2` and `4`,
//! a task summing `1` and `2` to get `3`,
//! and a task summing `3` and `4` to get the final output, `7`.
//!
//! A casual developer may write
//!
//! ```ignore
//! let _3 = sum(_1.await, _2.await).await;
//! let _7 = sum(_3, _4.await).await;
//! ```
//!
//! Above code is inefficient because every task only begins after the previous one completes.
//!
//! A better version would be
//!
//! ```ignore
//! let (_1, _2, _4) = join!(_1, _2, _4).await;
//! let _3 = sum(_1, _2).await;
//! let _7 = sum(_3, _4).await;
//! ```
//!
//! where `_1`, `_2` and `_4` run in parallel.
//!
//! However, above scheduling is still not optimal
//! because the summing of `_1` and `_2` can run in parallel with `_4`.
//!
//! To acheive maximum parallelism, one has to write something like
//!
//! ```ignore
//! let _1_2 = join!(_1, _2);
//! let (_3, _4) = select! {
//!     _3 = _1_2 => {
//!         (_3, _4.await)
//!     }
//!     _4 = _4 => {
//!         let (_1, _2) = _1_2.await;
//!         (sum(_1, _2).await, _4)
//!     }
//! }
//! let _7 = sum(_3, _4).await;
//! ```
//!
//! The code is quite obscure
//! and the manual scheduling quickly becomes tiring,
//! if possible at all, with a few more tasks and dependencies.
//!
//! With `async_dag`, one can write
//!
//! ```
//! use async_dag::Graph;
//!
//! async fn sum(lhs: i32, rhs: i32) -> i32 { lhs + rhs }
//!
//! async fn run() {
//!     let mut graph = Graph::new();
//!     // The closures are not run yet.
//!     let _1 = graph.add_task(|| async { 1 } );
//!     let _2 = graph.add_task(|| async { 2 } );
//!     let _4 = graph.add_task(|| async { 4 } );
//!
//!     // Sets `_1` as `_3`'s first parameter.
//!     let _3 = graph.add_child_task(_1, sum, 0).unwrap();
//!     // Sets `_2` as `_3`'s second parameter.
//!     graph.update_dependency(_2, _3, 1).unwrap();
//!
//!     // Sets `_3` as `_7`'s first parameter.
//!     let _7 = graph.add_child_task(_3, sum, 0).unwrap();
//!     // Sets `_4` as `_7`'s second parameter.
//!     graph.update_dependency(_4, _7, 1).unwrap();
//!
//!     // Runs all the tasks with maximum possible parallelism.
//!     graph.run().await;
//!
//!     assert_eq!(graph.get_value::<i32>(_7).unwrap(), 7);
//! }
//!
//! use futures::executor::block_on;
//! block_on(run());
//!
//! ```
//!
//! # Fail-fast graphs
//!
//! `TryGraph` can be used if the user wants a fail-fast strategy with fallible tasks.
//!
//! It aborts running futures when any one of them completes with a `Err`.

#![deny(warnings)]
#![warn(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_docs,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

mod any;
mod curry;
mod graph;
mod node;
mod task;
mod tuple;

pub use any::IntoAny;
pub use any::TypeInfo;
pub use curry::Curry;
pub use graph::*;
pub use task::{IntoInfallibleTask, IntoTryTask, TryTask};
