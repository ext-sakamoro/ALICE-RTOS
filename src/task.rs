//! Task definition — static, no-alloc task descriptors
//!
//! Each task is a periodic equation evaluator with fixed period,
//! WCET (worst-case execution time), and priority.
//!
//! Author: Moroya Sakamoto

/// Maximum tasks the kernel can manage
pub const MAX_TASKS: usize = 16;

/// Task function pointer — called each period
pub type TaskFn = fn(&mut [u8]);

/// Task priority (lower number = higher priority)
///
/// Rate-Monotonic: priority = 1 / period
/// Synth (44.1kHz, period=23µs) > Motion (10kHz, 100µs) > Edge (1kHz, 1ms)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(pub u8);

impl TaskPriority {
    /// Highest priority (audio rate tasks)
    pub const CRITICAL: TaskPriority = TaskPriority(0);
    /// High priority (motion control)
    pub const HIGH: TaskPriority = TaskPriority(1);
    /// Normal priority (sensor processing)
    pub const NORMAL: TaskPriority = TaskPriority(2);
    /// Low priority (logging, telemetry)
    pub const LOW: TaskPriority = TaskPriority(3);
    /// Background (non-real-time)
    pub const IDLE: TaskPriority = TaskPriority(255);
}

/// Task execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run
    Ready,
    /// Task is currently executing
    Running,
    /// Task is waiting for next period
    Sleeping,
    /// Task is suspended
    Suspended,
    /// Task slot is empty
    Inactive,
}

/// Static task descriptor — 32 bytes, no heap
#[derive(Clone, Copy)]
pub struct Task {
    /// Task name (8 ASCII chars max)
    pub name: [u8; 8],
    /// Task function pointer
    pub func: Option<TaskFn>,
    /// Priority (lower = higher priority)
    pub priority: TaskPriority,
    /// Period in microseconds
    pub period_us: u32,
    /// Worst-case execution time in microseconds
    pub wcet_us: u32,
    /// Current state
    pub state: TaskState,
    /// Next activation tick (absolute)
    pub next_activation: u64,
    /// Execution count
    pub exec_count: u32,
    /// Deadline miss count
    pub deadline_misses: u32,
    /// Scratch buffer size (bytes in shared scratch space)
    pub scratch_size: u16,
}

impl Task {
    /// Empty task slot
    pub const fn empty() -> Self {
        Self {
            name: [0u8; 8],
            func: None,
            priority: TaskPriority::IDLE,
            period_us: 0,
            wcet_us: 0,
            state: TaskState::Inactive,
            next_activation: 0,
            exec_count: 0,
            deadline_misses: 0,
            scratch_size: 0,
        }
    }

    /// Create a new periodic task
    pub fn new(name: &[u8], func: TaskFn, priority: TaskPriority, period_us: u32, wcet_us: u32) -> Self {
        let mut n = [0u8; 8];
        let len = name.len().min(8);
        n[..len].copy_from_slice(&name[..len]);

        Self {
            name: n,
            func: Some(func),
            priority,
            period_us,
            wcet_us,
            state: TaskState::Ready,
            next_activation: 0,
            exec_count: 0,
            deadline_misses: 0,
            scratch_size: 0,
        }
    }

    /// Is this task slot active?
    pub fn is_active(&self) -> bool {
        self.state != TaskState::Inactive
    }

    /// Frequency in Hz
    pub fn frequency_hz(&self) -> f32 {
        if self.period_us == 0 {
            0.0
        } else {
            1_000_000.0 / self.period_us as f32
        }
    }

    /// CPU utilization for this task
    pub fn utilization(&self) -> f32 {
        if self.period_us == 0 {
            0.0
        } else {
            self.wcet_us as f32 / self.period_us as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_task(_scratch: &mut [u8]) {}

    #[test]
    fn test_task_creation() {
        let task = Task::new(b"synth", dummy_task, TaskPriority::CRITICAL, 23, 10);
        assert!(task.is_active());
        assert_eq!(task.priority, TaskPriority::CRITICAL);
        assert_eq!(task.period_us, 23);
    }

    #[test]
    fn test_empty_task() {
        let task = Task::empty();
        assert!(!task.is_active());
        assert_eq!(task.state, TaskState::Inactive);
    }

    #[test]
    fn test_frequency() {
        let task = Task::new(b"motion", dummy_task, TaskPriority::HIGH, 100, 50);
        let freq = task.frequency_hz();
        assert!((freq - 10000.0).abs() < 1.0);
    }

    #[test]
    fn test_utilization() {
        let task = Task::new(b"edge", dummy_task, TaskPriority::NORMAL, 1000, 100);
        let u = task.utilization();
        assert!((u - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(TaskPriority::CRITICAL < TaskPriority::HIGH);
        assert!(TaskPriority::HIGH < TaskPriority::NORMAL);
        assert!(TaskPriority::NORMAL < TaskPriority::LOW);
    }
}
