//! System timer — hardware-abstract tick source
//!
//! Provides microsecond-resolution timing for the scheduler.
//! On real hardware, this wraps `SysTick` (Cortex-M) or MTIME (RISC-V).
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
    /// `clock_hz`: hardware clock frequency (e.g. `150_000_000` for Pi 5)
    #[must_use]
    pub const fn new(clock_hz: u32) -> Self {
        Self {
            ticks_us: 0,
            ticks_per_us: clock_hz / 1_000_000,
            overflows: 0,
        }
    }

    /// Software timer for testing
    #[must_use]
    pub const fn software() -> Self {
        Self {
            ticks_us: 0,
            ticks_per_us: 1,
            overflows: 0,
        }
    }

    /// Advance time by microseconds
    pub const fn advance(&mut self, us: u64) {
        let new = self.ticks_us.wrapping_add(us);
        if new < self.ticks_us {
            self.overflows += 1;
        }
        self.ticks_us = new;
    }

    /// Current time in microseconds
    #[must_use]
    pub const fn now_us(&self) -> u64 {
        self.ticks_us
    }

    /// Current time in milliseconds
    #[must_use]
    pub const fn now_ms(&self) -> u64 {
        self.ticks_us / 1000
    }

    /// Current time in seconds (as f32)
    #[must_use]
    pub fn now_secs(&self) -> f32 {
        self.ticks_us as f32 / 1_000_000.0
    }

    /// Reset timer
    pub const fn reset(&mut self) {
        self.ticks_us = 0;
        self.overflows = 0;
    }

    /// Timer frequency (ticks per microsecond)
    #[must_use]
    pub const fn ticks_per_us(&self) -> u32 {
        self.ticks_per_us
    }

    /// Number of overflows
    #[must_use]
    pub const fn overflows(&self) -> u32 {
        self.overflows
    }

    /// Elapsed microseconds since a reference point
    #[must_use]
    pub const fn elapsed_since(&self, reference: u64) -> u64 {
        self.ticks_us.wrapping_sub(reference)
    }

    /// Delay for given microseconds (busy-wait, for software timer)
    pub const fn delay_us(&mut self, us: u64) {
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
    #[must_use]
    pub const fn new(start: u64, period_us: u32) -> Self {
        Self {
            start,
            deadline: start + period_us as u64,
        }
    }

    /// Check if deadline is met
    #[must_use]
    pub const fn is_met(&self, current: u64) -> bool {
        current <= self.deadline
    }

    /// Remaining time until deadline (0 if missed)
    #[must_use]
    pub const fn remaining(&self, current: u64) -> u64 {
        self.deadline.saturating_sub(current)
    }

    /// Elapsed since start
    #[must_use]
    pub const fn elapsed(&self, current: u64) -> u64 {
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

    // --- 追加テスト ---

    #[test]
    fn test_hardware_timer_ticks_per_us() {
        // 72 MHz → 72 ticks/µs
        let timer = SysTimer::new(72_000_000);
        assert_eq!(timer.ticks_per_us(), 72);
    }

    #[test]
    fn test_software_timer_ticks_per_us() {
        let timer = SysTimer::software();
        assert_eq!(timer.ticks_per_us(), 1);
    }

    #[test]
    fn test_timer_advance_multiple_times() {
        let mut timer = SysTimer::software();
        timer.advance(100);
        timer.advance(200);
        timer.advance(300);
        assert_eq!(timer.now_us(), 600);
    }

    #[test]
    fn test_timer_now_ms_truncates() {
        let mut timer = SysTimer::software();
        timer.advance(1_999); // 1.999ms → 1ms（切り捨て）
        assert_eq!(timer.now_ms(), 1);
    }

    #[test]
    fn test_timer_now_ms_exact() {
        let mut timer = SysTimer::software();
        timer.advance(5_000);
        assert_eq!(timer.now_ms(), 5);
    }

    #[test]
    fn test_timer_now_secs_zero() {
        let timer = SysTimer::software();
        assert!(timer.now_secs() < f32::EPSILON);
    }

    #[test]
    fn test_timer_reset_clears_overflows() {
        let mut timer = SysTimer::software();
        timer.advance(u64::MAX); // オーバーフロー
        timer.reset();
        assert_eq!(timer.overflows(), 0);
        assert_eq!(timer.now_us(), 0);
    }

    #[test]
    fn test_timer_overflows_increments() {
        let mut timer = SysTimer::software();
        // wrapping_add で桁あふれを起こす
        timer.advance(u64::MAX);
        timer.advance(1);
        assert!(timer.overflows() >= 1);
    }

    #[test]
    fn test_timer_delay_us_advances_time() {
        let mut timer = SysTimer::software();
        timer.delay_us(250);
        assert_eq!(timer.now_us(), 250);
    }

    #[test]
    fn test_deadline_elapsed() {
        let dl = Deadline::new(1000, 500);
        assert_eq!(dl.elapsed(1000), 0);
        assert_eq!(dl.elapsed(1200), 200);
        assert_eq!(dl.elapsed(1500), 500);
    }

    #[test]
    fn test_deadline_remaining_zero_when_past() {
        let dl = Deadline::new(0, 100);
        // saturating_sub → 過ぎた後は 0
        assert_eq!(dl.remaining(200), 0);
        assert_eq!(dl.remaining(u64::MAX), 0);
    }

    #[test]
    fn test_deadline_at_boundary() {
        let dl = Deadline::new(500, 300);
        // deadline = 800
        assert!(dl.is_met(800));
        assert!(!dl.is_met(801));
    }

    #[test]
    fn test_elapsed_since_wrapping() {
        let timer = SysTimer::software();
        // timer は 0 から始まる。wrapping_sub(MAX) = 1
        let ref_time = u64::MAX;
        assert_eq!(timer.elapsed_since(ref_time), 1);
    }
}
