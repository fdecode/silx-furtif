use std::sync::Arc;

use tokio::sync::Mutex;
use slab::Slab;

/// Id builder for task request between servants and cluster master; this is an alias for a task id generator shared between threads
pub type IdBuilder = Arc<Mutex<TaskIdGenerator>>;

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
/// Task id to be used for task request between servants and cluster master
pub struct TaskId(AcknowledgeId);

/// Task id to be used for task request between servants and cluster master
pub type AcknowledgeId = u32;

impl TaskId {
    /// Get the acknowledgement id related to the task id
    pub fn acknowledge_id(&self) -> AcknowledgeId { self.0 }

    /// Constructor of task id
    pub (crate) fn new(a: AcknowledgeId) -> Self { Self(a) }
}

#[derive(Clone)]
/// Task id generator for task request between servants and cluster master
pub struct TaskIdGenerator(Slab<()>);

impl TaskIdGenerator {
    pub(crate) fn new() -> IdBuilder { Arc::new(Mutex::new(Self(Slab::new()))) }

    /// Generate a new task id
    /// * Output: a new task id
    pub fn generate(&mut self) -> TaskId { TaskId::new(self.0.insert(()) as AcknowledgeId) }

    pub(crate) fn delete(&mut self, TaskId(id): TaskId) -> Result<(),String> { 
        if self.0.try_remove(id as usize).is_none() { Err("TaskId: deleting a free identifier".to_string()) } else { Ok(()) }    
    }
}
