//! ALICE-Synth task templates for RTOS scheduling
//!
//! Pre-configured task descriptors for real-time audio synthesis.
//! Typical: 44.1 kHz sample generation (22.7 µs period).
//!
//! Author: Moroya Sakamoto

use crate::task::{Task, TaskFn, TaskPriority};

/// Default period for 44.1 kHz audio (≈ 22.7 µs)
pub const SYNTH_PERIOD_US: u32 = 23;

/// Default WCET for 4-voice FM synthesis on Cortex-M4F @ 168 MHz
pub const SYNTH_WCET_US: u32 = 8;

/// Default priority for synth tasks (critical — highest)
pub const SYNTH_PRIORITY: TaskPriority = TaskPriority::CRITICAL;

/// Period for 48 kHz sample rate (≈ 20.8 µs)
pub const SYNTH_48K_PERIOD_US: u32 = 21;

/// Period for 22.05 kHz sample rate (≈ 45.4 µs)
pub const SYNTH_22K_PERIOD_US: u32 = 45;

/// Create an ALICE-Synth task at 44.1 kHz with default WCET
///
/// - Period: 23 µs (44.1 kHz)
/// - WCET: 8 µs (4-voice FM)
/// - Priority: CRITICAL (0)
pub fn synth_task_default(func: TaskFn) -> Task {
    Task::new(
        b"synth",
        func,
        SYNTH_PRIORITY,
        SYNTH_PERIOD_US,
        SYNTH_WCET_US,
    )
}

/// Create an ALICE-Synth task with custom sample rate and WCET
pub fn synth_task(func: TaskFn, period_us: u32, wcet_us: u32) -> Task {
    Task::new(b"synth", func, SYNTH_PRIORITY, period_us, wcet_us)
}

/// Create an ALICE-Synth task at 48 kHz
pub fn synth_task_48k(func: TaskFn, wcet_us: u32) -> Task {
    Task::new(
        b"syn48k",
        func,
        SYNTH_PRIORITY,
        SYNTH_48K_PERIOD_US,
        wcet_us,
    )
}

/// Maximum voices sustainable at a given sample rate and CPU budget
///
/// Assumes `wcet_per_voice` µs per voice.
pub fn max_voices(period_us: u32, wcet_per_voice: u32) -> u32 {
    if wcet_per_voice == 0 {
        return 0;
    }
    period_us / wcet_per_voice
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_synth(_: &mut [u8]) {}

    #[test]
    fn test_synth_task_default() {
        let task = synth_task_default(dummy_synth);
        assert_eq!(task.period_us, SYNTH_PERIOD_US);
        assert_eq!(task.wcet_us, SYNTH_WCET_US);
        assert_eq!(task.priority, SYNTH_PRIORITY);
    }

    #[test]
    fn test_synth_task_48k() {
        let task = synth_task_48k(dummy_synth, 6);
        assert_eq!(task.period_us, SYNTH_48K_PERIOD_US);
        assert_eq!(task.wcet_us, 6);
    }

    #[test]
    fn test_synth_task_custom() {
        let task = synth_task(dummy_synth, 45, 15);
        assert_eq!(task.period_us, 45);
        assert_eq!(task.wcet_us, 15);
    }

    #[test]
    fn test_max_voices() {
        assert_eq!(max_voices(23, 2), 11);
        assert_eq!(max_voices(23, 8), 2);
        assert_eq!(max_voices(23, 0), 0);
    }
}
