use core::{pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod simple_executor;

pub struct Task {
    /*
        Syntax tip: dyn indicates we store a trait in the Box.
        This means that methods on the future are dynalically dispatched,
        allowing different types of futures to be stored in the same Task struct.

        Pin<Box> ensures value cannot be moved in memory by placing it on the heap
        and preventing the creation of &mut references to it.(Async/await might be self refential)
    */
    future: Pin<Box<dyn Future<Output = ()>>>, 
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}