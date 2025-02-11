use super::Task;
use alloc::collections::VecDeque;

pub struct SimpleExec {
    task_queue: VecDeque<Task>,
}

impl SimpleExec {
    pub fn new() -> Self {
        SimpleExec {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }
}
