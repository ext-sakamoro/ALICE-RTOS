//! ALICE-Motion task templates for RTOS scheduling
//!
//! Pre-configured task descriptors for real-time trajectory evaluation.
//! Typical: 10 kHz NURBS/spline evaluation for motor control.
//!
//! Author: Moroya Sakamoto

use crate::task::{Task, TaskFn, TaskPriority};

/// Default period for 10 kHz motion control (100 µs)
pub const MOTION_PERIOD_US: u32 = 100;

/// Default WCET for 3-DOF NURBS eval on Cortex-M4F @ 168 MHz
pub const MOTION_WCET_US: u32 = 15;

/// Default priority for motion tasks (high — above sensors, below audio)
pub const MOTION_PRIORITY: TaskPriority = TaskPriority::HIGH;

/// Period for 1 kHz servo control (1000 µs)
pub const MOTION_SERVO_PERIOD_US: u32 = 1_000;

/// Period for 50 kHz stepper control (20 µs)
pub const MOTION_STEPPER_PERIOD_US: u32 = 20;

/// Create an ALICE-Motion task at 10 kHz with default WCET
///
/// - Period: 100 µs (10 kHz)
/// - WCET: 15 µs (3-DOF NURBS)
/// - Priority: HIGH (1)
pub fn motion_task_default(func: TaskFn) -> Task {
    Task::new(
        b"motion",
        func,
        MOTION_PRIORITY,
        MOTION_PERIOD_US,
        MOTION_WCET_US,
    )
}

/// Create an ALICE-Motion task with custom parameters
pub fn motion_task(func: TaskFn, period_us: u32, wcet_us: u32) -> Task {
    Task::new(b"motion", func, MOTION_PRIORITY, period_us, wcet_us)
}

/// Create a servo control task (1 kHz, lower WCET budget)
pub fn motion_task_servo(func: TaskFn, wcet_us: u32) -> Task {
    Task::new(
        b"servo",
        func,
        MOTION_PRIORITY,
        MOTION_SERVO_PERIOD_US,
        wcet_us,
    )
}

/// Create a stepper motor task (50 kHz, critical priority)
pub fn motion_task_stepper(func: TaskFn, wcet_us: u32) -> Task {
    Task::new(
        b"stepper",
        func,
        TaskPriority::CRITICAL,
        MOTION_STEPPER_PERIOD_US,
        wcet_us,
    )
}

/// Calculate maximum DOF sustainable at a given rate and per-DOF WCET
pub fn max_dof(period_us: u32, wcet_per_dof: u32) -> u32 {
    if wcet_per_dof == 0 {
        return 0;
    }
    period_us / wcet_per_dof
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_motion(_: &mut [u8]) {}

    #[test]
    fn test_motion_task_default() {
        let task = motion_task_default(dummy_motion);
        assert_eq!(task.period_us, MOTION_PERIOD_US);
        assert_eq!(task.wcet_us, MOTION_WCET_US);
        assert_eq!(task.priority, MOTION_PRIORITY);
    }

    #[test]
    fn test_motion_task_servo() {
        let task = motion_task_servo(dummy_motion, 200);
        assert_eq!(task.period_us, MOTION_SERVO_PERIOD_US);
        assert_eq!(task.wcet_us, 200);
    }

    #[test]
    fn test_motion_task_stepper() {
        let task = motion_task_stepper(dummy_motion, 5);
        assert_eq!(task.period_us, MOTION_STEPPER_PERIOD_US);
        assert_eq!(task.priority, TaskPriority::CRITICAL);
    }

    #[test]
    fn test_motion_task_custom() {
        let task = motion_task(dummy_motion, 50, 10);
        assert_eq!(task.period_us, 50);
        assert_eq!(task.wcet_us, 10);
    }

    #[test]
    fn test_max_dof() {
        assert_eq!(max_dof(100, 15), 6);
        assert_eq!(max_dof(100, 0), 0);
    }
}
