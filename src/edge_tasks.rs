//! ALICE-Edge task templates for RTOS scheduling
//!
//! Pre-configured task descriptors for edge ML inference workloads.
//! Typical: 1 kHz evaluation of linear/polynomial models on sensor data.
//!
//! Author: Moroya Sakamoto

use crate::task::{Task, TaskFn, TaskPriority};

/// Default period for edge inference (1 kHz = 1000 µs)
pub const EDGE_PERIOD_US: u32 = 1_000;

/// Default WCET for 100-sample linear fit on Cortex-M0+ @ 133 MHz
pub const EDGE_WCET_US: u32 = 50;

/// Default priority for edge tasks (normal — below audio/motion)
pub const EDGE_PRIORITY: TaskPriority = TaskPriority::NORMAL;

/// Create an ALICE-Edge task with default parameters
///
/// - Period: 1 kHz (1000 µs)
/// - WCET: 50 µs
/// - Priority: NORMAL (2)
pub fn edge_task_default(func: TaskFn) -> Task {
    Task::new(b"edge", func, EDGE_PRIORITY, EDGE_PERIOD_US, EDGE_WCET_US)
}

/// Create an ALICE-Edge task with custom parameters
pub fn edge_task(func: TaskFn, period_us: u32, wcet_us: u32) -> Task {
    Task::new(b"edge", func, EDGE_PRIORITY, period_us, wcet_us)
}

/// Create an ALICE-Edge task for high-frequency inference (10 kHz)
pub fn edge_task_fast(func: TaskFn) -> Task {
    Task::new(b"edge_hf", func, TaskPriority::HIGH, 100, 20)
}

/// Estimate CPU utilization for an edge task configuration
pub fn edge_utilization(period_us: u32, wcet_us: u32) -> f32 {
    if period_us == 0 {
        return 0.0;
    }
    wcet_us as f32 / period_us as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_edge(_: &mut [u8]) {}

    #[test]
    fn test_edge_task_default() {
        let task = edge_task_default(dummy_edge);
        assert_eq!(task.period_us, EDGE_PERIOD_US);
        assert_eq!(task.wcet_us, EDGE_WCET_US);
        assert_eq!(task.priority, EDGE_PRIORITY);
        assert!(task.is_active());
    }

    #[test]
    fn test_edge_task_custom() {
        let task = edge_task(dummy_edge, 500, 30);
        assert_eq!(task.period_us, 500);
        assert_eq!(task.wcet_us, 30);
    }

    #[test]
    fn test_edge_task_fast() {
        let task = edge_task_fast(dummy_edge);
        assert_eq!(task.period_us, 100);
        assert_eq!(task.priority, TaskPriority::HIGH);
    }

    #[test]
    fn test_edge_utilization() {
        let u = edge_utilization(EDGE_PERIOD_US, EDGE_WCET_US);
        assert!((u - 0.05).abs() < 0.01);
        assert_eq!(edge_utilization(0, 50), 0.0);
    }
}
