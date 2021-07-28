use core::pin::Pin;
use crate::task::Task;

/// Executes `[Task]`s
pub trait Executor {
    /// Start executing a task or schedule its execution
    fn exec(&mut self, task: Pin<&mut Task>);
}
