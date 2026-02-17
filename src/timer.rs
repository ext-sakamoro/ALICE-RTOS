//! System timer — hardware-abstract tick source
//!
//! Provides microsecond-resolution timing for the scheduler.
//! On real hardware, this wraps SysTick (Cortex-M) or MTIME (RISC-V).
//! For testing, uses a software counter.
//!
//! Author: Moroya Sakamoto

/// System timer
///
/// Size: 16 bytes
pub struct SysTimer {
    /// Current tick count (microseconds)
    ticks_us: u64,
    /// Timer frequency (ticks per microsecond)
    ticks_per_us: u32,
    /// Overflow count
    overflows: u32,
}

impl SysTimer {
    /// Create a new system timer
    ///
    /// `clock_hz`: hardware clock frequency (e.g. 150_000_000 for Pi 5)
    pub const fn new(clock_hz: u32) -> Self {
        Self {
            ticks_us: 0,
            ticks_per_us: clock_hz / 1_000_000,
            overflows: 0,
        }
    }

    /// Software timer for testing
    pub const fn software() -> Self {
        Self {
            ticks_us: 0,
            ticks_per_us: 1,
            overflows: 0,
        }
    }

    /// Advance time by microseconds
    pub fn advance(&mut self, us: u64) {
        let new = self.ticks_us.wrapping_add(us);
        if new < self.ticks_us {
            self.overflows += 1;
        }
        self.ticks_us = new;
    }

    /// Current time in microseconds
    pub fn now_us(&self) -> u64 {
        self.ticks_us
    }

    /// Current time in milliseconds
    pub fn now_ms(&self) -> u64 {
        self.ticks_us / 1000
    }

    /// Current time in seconds (as f32)
    pub fn now_secs(&self) -> f32 {
        self.ticks_us as f32 / 1_000_000.0
    }

    /// Reset timer
    pub fn reset(&mut self) {
        self.ticks_us = 0;
        self.overflows = 0;
    }

    /// Timer frequency (ticks per microsecond)
    pub fn ticks_per_us(&self) -> u32 {
        self.ticks_per_us
    }

    /// Number of overflows
    pub fn overflows(&self) -> u32 {
        self.overflows
    }

    /// Elapsed microseconds since a reference point
    pub fn elapsed_since(&self, reference: u64) -> u64 {
        self.ticks_us.wrapping_sub(reference)
    }

    /// Delay for given microseconds (busy-wait, for software timer)
    pub fn delay_us(&mut self, us: u64) {
        self.advance(us);
    }
}

/// Deadline tracker for a single task
pub struct Deadline {
    /// Activation time
    start: u64,
    /// Deadline (absolute)
    deadline: u64,
}

impl Deadline {
    /// Create a new deadline
    pub fn new(start: u64, period_us: u32) -> Self {
        Self {
            start,
            deadline: start + period_us as u64,
        }
    }

    /// Check if deadline is met
    pub fn is_met(&self, current: u64) -> bool {
        current <= self.deadline
    }

    /// Remaining time until deadline (0 if missed)
    pub fn remaining(&self, current: u64) -> u64 {
        if current >= self.deadline {
            0
        } else {
            self.deadline - current
        }
    }

    /// Elapsed since start
    pub fn elapsed(&self, current: u64) -> u64 {
        current.wrapping_sub(self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_timer() {
        let mut timer = SysTimer::software();
        assert_eq!(timer.now_us(), 0);
        timer.advance(1000);
        assert_eq!(timer.now_us(), 1000);
        assert_eq!(timer.now_ms(), 1);
    }

    #[test]
    fn test_timer_secs() {
        let mut timer = SysTimer::software();
        timer.advance(1_500_000);
        assert!((timer.now_secs() - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = SysTimer::software();
        timer.advance(10000);
        timer.reset();
        assert_eq!(timer.now_us(), 0);
    }

    #[test]
    fn test_deadline_met() {
        let deadline = Deadline::new(0, 1000);
        assert!(deadline.is_met(500));
        assert!(deadline.is_met(1000));
        assert!(!deadline.is_met(1001));
    }

    #[test]
    fn test_deadline_remaining() {
        let deadline = Deadline::new(0, 1000);
        assert_eq!(deadline.remaining(500), 500);
        assert_eq!(deadline.remaining(1000), 0);
        assert_eq!(deadline.remaining(2000), 0);
    }

    #[test]
    fn test_elapsed_since() {
        let mut timer = SysTimer::software();
        timer.advance(1000);
        let ref_time = timer.now_us();
        timer.advance(500);
        assert_eq!(timer.elapsed_since(ref_time), 500);
    }
}
