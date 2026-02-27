use core::{pin::Pin, sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod keyboard;
pub mod executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    /*
        Syntax tip: dyn indicates we store a trait in the Box.
        This means that methods on the future are dynalically dispatched,
        allowing different types of futures to be stored in the same Task struct.

        Pin<Box> ensures value cannot be moved in memory by placing it on the heap
        and preventing the creation of &mut references to it.(Async/await might be self refential)
    */
    future: Pin<Box<dyn Future<Output = ()>>>, 
    id: TaskId,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}