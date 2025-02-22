use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::{Context, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Exec {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Exec {
    pub fn new() -> Self {
        Exec {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("Task with ID already in queue");
        }
        self.task_queue.push(task_id).expect("Warning: queue full");
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                // TODO: implement TaskWaker
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                core::task::Poll::Ready(()) => {
                    // task = finished --> remove task and waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                core::task::Poll::Pending => {}
            }
        }
    }
}
