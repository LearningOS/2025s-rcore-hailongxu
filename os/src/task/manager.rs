//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut iter = self.ready_queue.iter().enumerate();
        let Some((i,task)) = iter.next() else {
            return None;
        };
        let mut i_min = i;
        let mut stride_min = task.inner_exclusive_access().stride;

        for (i,e) in iter {
            let stride = e.inner_exclusive_access().stride;
            if (stride_min - stride) as isize > 0 {
                i_min = i;
                stride_min = stride;
            }
        }
        let Some(task) = self.ready_queue.remove(i_min) else {
            return None;
        };
        let mut task_mut = task.inner_exclusive_access();
        task_mut.stride += crate::config::BIG_STRIDE/task_mut.priority;
        drop(task_mut);
        Some(task)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

/// print all ready tasks
#[allow(dead_code)]
pub fn print_all_tasks() {
    let tasks = TASK_MANAGER.exclusive_access();
    let tasks = &tasks.ready_queue;
    for (i,task) in tasks.iter().enumerate() {
        let inner = task.inner_exclusive_access();
        println!("tid:{} i:{} {} {} {} {:?}",task.getpid(),i,"inner.name",inner.priority,inner.stride as isize, inner.task_status);
    }
}
