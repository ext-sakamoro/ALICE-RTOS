//! Kernel — top-level RTOS manager
//!
//! Combines scheduler + timer + scratch memory into a single
//! kernel instance. Entry point for ALICE-RTOS usage.
//!
//! Author: Moroya Sakamoto

use crate::scheduler::Scheduler;
use crate::timer::SysTimer;
use crate::task::{Task, TaskPriority, TaskFn};

/// Scratch buffer for task execution
const SCRATCH_SIZE: usize = 1024;

/// ALICE-RTOS Kernel
///
/// Total memory footprint:
/// - Scheduler: ~512 bytes (16 tasks × 32 bytes)
/// - Timer: 16 bytes
/// - Scratch: 1024 bytes
/// - Total: < 2 KB
pub struct Kernel {
    /// Task scheduler
    pub scheduler: Scheduler,
    /// System timer
    pub timer: SysTimer,
    /// Shared scratch buffer for task execution
    scratch: [u8; SCRATCH_SIZE],
    /// Kernel state
    running: bool,
    /// Total ticks executed
    pub total_ticks: u64,
}

impl Kernel {
    /// Create kernel with hardware clock
    pub const fn new(clock_hz: u32) -> Self {
        Self {
            scheduler: Scheduler::new(),
            timer: SysTimer::new(clock_hz),
            scratch: [0u8; SCRATCH_SIZE],
            running: false,
            total_ticks: 0,
        }
    }

    /// Create kernel for testing (software timer)
    pub const fn testing() -> Self {
        Self {
            scheduler: Scheduler::new(),
            timer: SysTimer::software(),
            scratch: [0u8; SCRATCH_SIZE],
            running: false,
            total_ticks: 0,
        }
    }

    /// Register a task
    pub fn add_task(
        &mut self,
        name: &[u8],
        func: TaskFn,
        priority: TaskPriority,
        period_us: u32,
        wcet_us: u32,
    ) -> Option<usize> {
        let task = Task::new(name, func, priority, period_us, wcet_us);
        self.scheduler.register(task)
    }

    /// Run one scheduler tick
    ///
    /// Advances time by `delta_us` and executes the highest-priority ready task.
    /// Returns the task index that was executed, if any.
    pub fn tick(&mut self, delta_us: u64) -> Option<usize> {
        self.timer.advance(delta_us);
        self.total_ticks += 1;

        let executed = self.scheduler.tick(delta_us);

        // Execute the task with scratch buffer
        if let Some(idx) = executed {
            self.scheduler.execute_task(idx, &mut self.scratch);
        }

        executed
    }

    /// Run the kernel for a given duration (testing)
    pub fn run_for(&mut self, total_us: u64, tick_us: u64) -> KernelStats {
        self.running = true;
        let mut elapsed = 0u64;
        let mut tasks_executed = 0u64;

        while elapsed < total_us && self.running {
            if self.tick(tick_us).is_some() {
                tasks_executed += 1;
            }
            elapsed += tick_us;
        }

        self.running = false;
        KernelStats {
            total_us: elapsed,
            total_ticks: self.total_ticks,
            tasks_executed,
            context_switches: self.scheduler.context_switches as u64,
            utilization: self.scheduler.total_utilization(),
            schedulable: self.scheduler.is_schedulable(),
        }
    }

    /// Stop the kernel
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Is the kernel running?
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Check RMS schedulability
    pub fn is_schedulable(&self) -> bool {
        self.scheduler.is_schedulable()
    }

    /// Memory footprint estimate
    pub fn memory_footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

/// Kernel execution statistics
#[derive(Debug, Clone)]
pub struct KernelStats {
    /// Total elapsed time (µs)
    pub total_us: u64,
    /// Total scheduler ticks
    pub total_ticks: u64,
    /// Tasks executed
    pub tasks_executed: u64,
    /// Context switches
    pub context_switches: u64,
    /// CPU utilization
    pub utilization: f32,
    /// RMS schedulable
    pub schedulable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_task(_scratch: &mut [u8]) {}

    #[test]
    fn test_kernel_creation() {
        let kernel = Kernel::testing();
        assert_eq!(kernel.scheduler.active_task_count(), 0);
    }

    #[test]
    fn test_kernel_add_task() {
        let mut kernel = Kernel::testing();
        let idx = kernel.add_task(b"test", noop_task, TaskPriority::NORMAL, 1000, 100);
        assert_eq!(idx, Some(0));
        assert_eq!(kernel.scheduler.active_task_count(), 1);
    }

    #[test]
    fn test_kernel_tick() {
        let mut kernel = Kernel::testing();
        kernel.add_task(b"test", noop_task, TaskPriority::NORMAL, 100, 10);

        let executed = kernel.tick(0);
        assert_eq!(executed, Some(0));
    }

    #[test]
    fn test_kernel_run_for() {
        let mut kernel = Kernel::testing();
        kernel.add_task(b"fast", noop_task, TaskPriority::HIGH, 100, 10);
        kernel.add_task(b"slow", noop_task, TaskPriority::LOW, 1000, 50);

        let stats = kernel.run_for(10_000, 10);
        assert!(stats.tasks_executed > 0);
        assert!(stats.schedulable);
    }

    #[test]
    fn test_kernel_schedulability() {
        let mut kernel = Kernel::testing();
        kernel.add_task(b"synth", noop_task, TaskPriority::CRITICAL, 23, 10);
        kernel.add_task(b"motion", noop_task, TaskPriority::HIGH, 100, 20);
        kernel.add_task(b"edge", noop_task, TaskPriority::NORMAL, 1000, 100);

        assert!(kernel.is_schedulable());
    }

    #[test]
    fn test_memory_footprint() {
        let kernel = Kernel::testing();
        let size = kernel.memory_footprint();
        // Should be under 2KB
        assert!(size < 2048, "kernel size should be < 2KB, got {size}");
    }

    #[test]
    fn test_kernel_stats() {
        let mut kernel = Kernel::testing();
        kernel.add_task(b"t1", noop_task, TaskPriority::NORMAL, 100, 10);
        let stats = kernel.run_for(1000, 100);
        assert_eq!(stats.total_us, 1000);
        assert!(stats.utilization > 0.0);
    }
}
