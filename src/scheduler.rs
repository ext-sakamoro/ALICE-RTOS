//! Rate-Monotonic Scheduler
//!
//! Fixed-priority preemptive scheduling with RMS schedulability analysis.
//! Guarantees: if total utilization ≤ n(2^(1/n) - 1), all deadlines are met.
//!
//! Author: Moroya Sakamoto

use crate::task::{Task, TaskState, TaskPriority, MAX_TASKS};

/// Rate-Monotonic Scheduler
///
/// Static task table, no dynamic allocation.
/// Size: MAX_TASKS × sizeof(Task) + overhead ≈ 512 + 32 bytes
pub struct Scheduler {
    /// Static task table
    tasks: [Task; MAX_TASKS],
    /// Number of registered tasks
    task_count: usize,
    /// Currently running task index (None = idle)
    current_task: Option<usize>,
    /// System tick counter (microseconds)
    tick_us: u64,
    /// Total context switches
    pub context_switches: u32,
}

impl Scheduler {
    /// Create empty scheduler
    pub const fn new() -> Self {
        Self {
            tasks: [Task::empty(); MAX_TASKS],
            task_count: 0,
            current_task: None,
            tick_us: 0,
            context_switches: 0,
        }
    }

    /// Register a task, returns slot index
    pub fn register(&mut self, task: Task) -> Option<usize> {
        if self.task_count >= MAX_TASKS {
            return None;
        }
        let idx = self.task_count;
        self.tasks[idx] = task;
        self.tasks[idx].next_activation = self.tick_us;
        self.task_count += 1;
        Some(idx)
    }

    /// Advance system time by `delta_us` microseconds and run ready tasks
    ///
    /// Returns the index of the task that was executed, if any.
    pub fn tick(&mut self, delta_us: u64) -> Option<usize> {
        self.tick_us += delta_us;

        // Mark tasks whose period has elapsed as Ready
        for i in 0..self.task_count {
            if self.tasks[i].state == TaskState::Sleeping
                && self.tick_us >= self.tasks[i].next_activation
            {
                self.tasks[i].state = TaskState::Ready;
            }
        }

        // Find highest-priority ready task
        let next = self.find_highest_priority_ready();

        if let Some(idx) = next {
            // Context switch?
            if self.current_task != Some(idx) {
                self.context_switches += 1;
                self.current_task = Some(idx);
            }

            // Check deadline
            if self.tick_us > self.tasks[idx].next_activation + self.tasks[idx].period_us as u64 {
                self.tasks[idx].deadline_misses += 1;
            }

            // Execute task
            self.tasks[idx].state = TaskState::Running;
            self.tasks[idx].exec_count += 1;

            // Schedule next activation
            self.tasks[idx].next_activation += self.tasks[idx].period_us as u64;
            self.tasks[idx].state = TaskState::Sleeping;

            Some(idx)
        } else {
            self.current_task = None;
            None
        }
    }

    /// Execute a specific task (call its function with scratch buffer)
    pub fn execute_task(&self, idx: usize, scratch: &mut [u8]) {
        if let Some(func) = self.tasks[idx].func {
            func(scratch);
        }
    }

    /// Find highest-priority (lowest number) ready task
    fn find_highest_priority_ready(&self) -> Option<usize> {
        let mut best_idx = None;
        let mut best_priority = TaskPriority::IDLE;

        for i in 0..self.task_count {
            if self.tasks[i].state == TaskState::Ready && self.tasks[i].priority < best_priority {
                best_priority = self.tasks[i].priority;
                best_idx = Some(i);
            }
        }
        best_idx
    }

    /// RMS schedulability test
    ///
    /// Liu & Layland bound: U ≤ n(2^(1/n) - 1)
    /// For n=3: U ≤ 0.780
    /// For n→∞: U ≤ ln(2) ≈ 0.693
    pub fn is_schedulable(&self) -> bool {
        let n = self.active_task_count();
        if n == 0 {
            return true;
        }
        let total_u = self.total_utilization();
        let bound = liu_layland_bound(n);
        total_u <= bound
    }

    /// Total CPU utilization (sum of Ci/Ti for all tasks)
    pub fn total_utilization(&self) -> f32 {
        let mut u = 0.0f32;
        for i in 0..self.task_count {
            if self.tasks[i].is_active() {
                u += self.tasks[i].utilization();
            }
        }
        u
    }

    /// Number of active tasks
    pub fn active_task_count(&self) -> usize {
        self.tasks[..self.task_count]
            .iter()
            .filter(|t| t.is_active())
            .count()
    }

    /// Get task by index
    pub fn get_task(&self, idx: usize) -> Option<&Task> {
        if idx < self.task_count {
            Some(&self.tasks[idx])
        } else {
            None
        }
    }

    /// Current system time in microseconds
    pub fn now_us(&self) -> u64 {
        self.tick_us
    }

    /// Suspend a task
    pub fn suspend(&mut self, idx: usize) {
        if idx < self.task_count {
            self.tasks[idx].state = TaskState::Suspended;
        }
    }

    /// Resume a suspended task
    pub fn resume(&mut self, idx: usize) {
        if idx < self.task_count && self.tasks[idx].state == TaskState::Suspended {
            self.tasks[idx].state = TaskState::Ready;
            self.tasks[idx].next_activation = self.tick_us;
        }
    }
}

/// Liu & Layland bound: n(2^(1/n) - 1)
///
/// Uses precomputed table for small n, approximation for large n.
fn liu_layland_bound(n: usize) -> f32 {
    // Precomputed bounds for common task counts
    const BOUNDS: [f32; 10] = [
        1.000, // n=0: unused (returns early)
        1.000, // n=1: U ≤ 1.000
        0.828, // n=2: U ≤ 0.828
        0.780, // n=3: U ≤ 0.780
        0.757, // n=4: U ≤ 0.757
        0.743, // n=5: U ≤ 0.743
        0.735, // n=6: U ≤ 0.735
        0.729, // n=7: U ≤ 0.729
        0.724, // n=8: U ≤ 0.724
        0.693, // n≥9: U ≤ ln(2) ≈ 0.693
    ];
    if n == 0 {
        return 1.0;
    }
    let idx = n.min(9);
    BOUNDS[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskPriority};

    fn dummy_task(_: &mut [u8]) {}

    #[test]
    fn test_scheduler_empty() {
        let sched = Scheduler::new();
        assert_eq!(sched.active_task_count(), 0);
        assert!(sched.is_schedulable());
    }

    #[test]
    fn test_register_task() {
        let mut sched = Scheduler::new();
        let task = Task::new(b"synth", dummy_task, TaskPriority::CRITICAL, 23, 10);
        let idx = sched.register(task);
        assert_eq!(idx, Some(0));
        assert_eq!(sched.active_task_count(), 1);
    }

    #[test]
    fn test_tick_executes_ready_task() {
        let mut sched = Scheduler::new();
        let task = Task::new(b"test", dummy_task, TaskPriority::NORMAL, 100, 10);
        sched.register(task);

        // First tick: task should be ready immediately
        let executed = sched.tick(0);
        assert_eq!(executed, Some(0));
    }

    #[test]
    fn test_priority_order() {
        let mut sched = Scheduler::new();
        let low = Task::new(b"low", dummy_task, TaskPriority::LOW, 1000, 100);
        let high = Task::new(b"high", dummy_task, TaskPriority::HIGH, 100, 50);
        sched.register(low);
        sched.register(high);

        // Both ready → high priority should execute first
        let executed = sched.tick(0);
        assert_eq!(executed, Some(1)); // high is at index 1
    }

    #[test]
    fn test_periodic_execution() {
        let mut sched = Scheduler::new();
        let task = Task::new(b"periodic", dummy_task, TaskPriority::NORMAL, 100, 10);
        sched.register(task);

        // Execute at t=0
        sched.tick(0);
        // Advance time to next period
        let executed = sched.tick(100);
        assert_eq!(executed, Some(0));
        assert_eq!(sched.get_task(0).unwrap().exec_count, 2);
    }

    #[test]
    fn test_schedulability() {
        let mut sched = Scheduler::new();
        // Three tasks with total U = 0.1 + 0.5 + 0.1 = 0.7 < 0.780 (n=3 bound)
        sched.register(Task::new(b"t1", dummy_task, TaskPriority::CRITICAL, 100, 10));
        sched.register(Task::new(b"t2", dummy_task, TaskPriority::HIGH, 100, 50));
        sched.register(Task::new(b"t3", dummy_task, TaskPriority::NORMAL, 1000, 100));
        assert!(sched.is_schedulable());
    }

    #[test]
    fn test_overloaded_not_schedulable() {
        let mut sched = Scheduler::new();
        // Two tasks with total U = 0.9 + 0.5 = 1.4 > 0.828 (n=2 bound)
        sched.register(Task::new(b"t1", dummy_task, TaskPriority::CRITICAL, 100, 90));
        sched.register(Task::new(b"t2", dummy_task, TaskPriority::HIGH, 100, 50));
        assert!(!sched.is_schedulable());
    }

    #[test]
    fn test_suspend_resume() {
        let mut sched = Scheduler::new();
        sched.register(Task::new(b"test", dummy_task, TaskPriority::NORMAL, 100, 10));
        sched.suspend(0);
        assert_eq!(sched.get_task(0).unwrap().state, TaskState::Suspended);

        let executed = sched.tick(100);
        assert!(executed.is_none()); // Suspended tasks don't run

        sched.resume(0);
        let executed = sched.tick(0);
        assert_eq!(executed, Some(0));
    }

    #[test]
    fn test_liu_layland() {
        assert!((liu_layland_bound(1) - 1.0).abs() < 0.01);
        assert!((liu_layland_bound(3) - 0.780).abs() < 0.01);
    }
}
