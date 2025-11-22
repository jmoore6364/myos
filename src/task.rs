use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::context::Context;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);

const STACK_SIZE: usize = 8192; // 8KB stack per task

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
    stack: Box<[u8]>,
    context: Context,
}

impl Task {
    /// Create a new task with the given name, priority, and entry point function
    pub fn new(name: String, priority: u8, entry_point: extern "C" fn()) -> Self {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);

        // Allocate stack
        let stack = vec![0u8; STACK_SIZE].into_boxed_slice();

        // Get stack top (stacks grow downward)
        let stack_top = stack.as_ptr() as u64 + STACK_SIZE as u64;

        // Initialize context
        let mut context = Context::new();
        context.init(entry_point, stack_top);

        Task {
            id,
            name,
            state: TaskState::Ready,
            priority,
            stack,
            context,
        }
    }

    /// Create an idle task (just halts in a loop)
    pub fn new_idle() -> Self {
        extern "C" fn idle_task() {
            loop {
                x86_64::instructions::hlt();
            }
        }

        Self::new(String::from("idle"), 0, idle_task)
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

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn context(&self) -> &Context {
        &self.context
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

    pub fn preemptive_schedule(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        // Find next ready task
        let start_index = self.current_task.map(|i| (i + 1) % self.tasks.len()).unwrap_or(0);
        let mut next_index = None;

        for offset in 0..self.tasks.len() {
            let index = (start_index + offset) % self.tasks.len();
            if self.tasks[index].state() == TaskState::Ready {
                next_index = Some(index);
                break;
            }
        }

        // If we found a task to switch to, perform context switch
        if let Some(next) = next_index {
            if Some(next) != self.current_task {
                // Perform context switch
                match self.current_task {
                    Some(current) if current < self.tasks.len() => {
                        // Mark current task as ready
                        if self.tasks[current].state() == TaskState::Running {
                            self.tasks[current].set_state(TaskState::Ready);
                        }

                        // Get context pointers
                        let old_ctx = self.tasks[current].context_mut() as *mut Context;
                        let new_ctx = self.tasks[next].context() as *const Context;

                        // Mark new task as running
                        self.tasks[next].set_state(TaskState::Running);
                        self.current_task = Some(next);

                        // Perform the actual context switch
                        unsafe {
                            crate::context::switch_context(old_ctx, new_ctx);
                        }
                    }
                    _ => {
                        // No current task, just start the next one
                        self.tasks[next].set_state(TaskState::Running);
                        self.current_task = Some(next);
                    }
                }
            }
        }
    }

    /// Switch to a specific task by ID (for testing)
    pub fn switch_to(&mut self, task_id: u64) -> Result<(), &'static str> {
        let task_index = self.tasks.iter().position(|t| t.id() == task_id)
            .ok_or("Task not found")?;

        if self.tasks[task_index].state() == TaskState::Terminated {
            return Err("Task is terminated");
        }

        match self.current_task {
            Some(current) if current < self.tasks.len() => {
                self.tasks[current].set_state(TaskState::Ready);

                let old_ctx = self.tasks[current].context_mut() as *mut Context;
                let new_ctx = self.tasks[task_index].context() as *const Context;

                self.tasks[task_index].set_state(TaskState::Running);
                self.current_task = Some(task_index);

                unsafe {
                    crate::context::switch_context(old_ctx, new_ctx);
                }
            }
            _ => {
                self.tasks[task_index].set_state(TaskState::Running);
                self.current_task = Some(task_index);
            }
        }

        Ok(())
    }
}

// Global function called from timer interrupt
pub fn schedule() {
    // NOTE: Context switching from interrupt handlers is tricky because of the
    // interrupt stack frame. This implementation attempts preemptive scheduling.
    // If issues arise, the manual switch_to() method can be used instead.

    let mut scheduler = crate::SCHEDULER.lock();

    if scheduler.tasks.is_empty() {
        return;
    }

    // Perform preemptive context switch
    scheduler.preemptive_schedule();
}
