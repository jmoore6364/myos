use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Waiting,
    Terminated,
}

pub struct Task {
    id: u64,
    name: String,
    state: TaskState,
    priority: u8,
    stack: Vec<u8>,
    instruction_pointer: usize,
}

impl Task {
    pub fn new(name: String, priority: u8) -> Self {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        Task {
            id,
            name,
            state: TaskState::Ready,
            priority,
            stack: Vec::with_capacity(4096), // 4KB stack per task
            instruction_pointer: 0,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }
}

pub struct Scheduler {
    tasks: Vec<Task>,
    current_task: Option<usize>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            tasks: Vec::new(),
            current_task: None,
        }
    }

    pub fn add_task(&mut self, task: Task) -> u64 {
        let id = task.id();
        self.tasks.push(task);
        id
    }

    pub fn remove_task(&mut self, id: u64) -> Result<(), &'static str> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id() == id) {
            self.tasks.remove(pos);
            Ok(())
        } else {
            Err("Task not found")
        }
    }

    pub fn get_task(&self, id: u64) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id() == id)
    }

    pub fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn schedule_next(&mut self) -> Option<&mut Task> {
        if self.tasks.is_empty() {
            return None;
        }

        // Simple round-robin scheduling
        let start_index = self.current_task.map(|i| (i + 1) % self.tasks.len()).unwrap_or(0);

        for offset in 0..self.tasks.len() {
            let index = (start_index + offset) % self.tasks.len();
            if self.tasks[index].state() == TaskState::Ready {
                self.current_task = Some(index);
                self.tasks[index].set_state(TaskState::Running);
                return Some(&mut self.tasks[index]);
            }
        }

        None
    }

    pub fn yield_current(&mut self) {
        if let Some(index) = self.current_task {
            if self.tasks[index].state() == TaskState::Running {
                self.tasks[index].set_state(TaskState::Ready);
            }
        }
    }

    pub fn terminate_current(&mut self) {
        if let Some(index) = self.current_task {
            self.tasks[index].set_state(TaskState::Terminated);
            self.current_task = None;
        }
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn ready_task_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state() == TaskState::Ready).count()
    }
}
