use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

pub mod simple_exec;

// type wrapper for a pinned, heap allocated, dynamically sized Future value for some Task t
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
