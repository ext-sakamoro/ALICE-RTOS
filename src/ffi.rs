//! C-ABI FFI bindings for ALICE-RTOS
//!
//! 66 `extern "C"` functions for Unity/UE5 integration.
//! Enables RTOS scheduling simulation and monitoring from game engines.
//!
//! Prefix: `ar_rtos_*`
//!
//! Author: Moroya Sakamoto

use crate::kernel::{Kernel, KernelStats};
use crate::scheduler::Scheduler;
use crate::spsc::SpscRing;
use crate::task::{TaskPriority, TaskState};
use crate::timer::{Deadline, SysTimer};

// ============================================================================
// Opaque handle types
// ============================================================================

pub struct ArRtosKernel(Kernel);
pub struct ArRtosScheduler(Scheduler);
pub struct ArRtosTimer(SysTimer);
pub struct ArRtosSpsc(SpscRing<256>);
pub struct ArRtosStats(KernelStats);

/// No-op task function for FFI-registered tasks
fn ffi_noop_task(_: &mut [u8]) {}

// ============================================================================
// Kernel (12 functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_kernel_new(clock_hz: u32) -> *mut ArRtosKernel {
    Box::into_raw(Box::new(ArRtosKernel(Kernel::new(clock_hz))))
}

#[no_mangle]
pub extern "C" fn ar_rtos_kernel_testing() -> *mut ArRtosKernel {
    Box::into_raw(Box::new(ArRtosKernel(Kernel::testing())))
}

/// # Safety
/// `ptr` must be a valid pointer from `ar_rtos_kernel_new` or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_free(ptr: *mut ArRtosKernel) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// `ptr` and `name_ptr` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_add_task(
    ptr: *mut ArRtosKernel,
    name_ptr: *const u8,
    name_len: u32,
    priority: u8,
    period_us: u32,
    wcet_us: u32,
) -> i32 {
    if ptr.is_null() || name_ptr.is_null() {
        return -1;
    }
    let kernel = &mut (*ptr).0;
    let name = core::slice::from_raw_parts(name_ptr, name_len as usize);
    match kernel.add_task(
        name,
        ffi_noop_task,
        TaskPriority(priority),
        period_us,
        wcet_us,
    ) {
        Some(idx) => idx as i32,
        None => -1,
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_tick(ptr: *mut ArRtosKernel, delta_us: u64) -> i32 {
    if ptr.is_null() {
        return -1;
    }
    match (*ptr).0.tick(delta_us) {
        Some(idx) => idx as i32,
        None => -1,
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_run_for(
    ptr: *mut ArRtosKernel,
    total_us: u64,
    tick_us: u64,
) -> *mut ArRtosStats {
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let stats = (*ptr).0.run_for(total_us, tick_us);
    Box::into_raw(Box::new(ArRtosStats(stats)))
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_stop(ptr: *mut ArRtosKernel) {
    if !ptr.is_null() {
        (*ptr).0.stop();
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_is_running(ptr: *const ArRtosKernel) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    i32::from((*ptr).0.is_running())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_is_schedulable(ptr: *const ArRtosKernel) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    i32::from((*ptr).0.is_schedulable())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_memory_footprint(ptr: *const ArRtosKernel) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.memory_footprint() as u32
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_total_ticks(ptr: *const ArRtosKernel) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.total_ticks
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_kernel_active_task_count(ptr: *const ArRtosKernel) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.scheduler.active_task_count() as u32
}

// ============================================================================
// KernelStats (7 functions)
// ============================================================================

/// # Safety
/// `ptr` must be a valid pointer from `ar_rtos_kernel_run_for` or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_free(ptr: *mut ArRtosStats) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_total_us(ptr: *const ArRtosStats) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.total_us
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_total_ticks(ptr: *const ArRtosStats) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.total_ticks
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_tasks_executed(ptr: *const ArRtosStats) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.tasks_executed
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_context_switches(ptr: *const ArRtosStats) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.context_switches
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_utilization(ptr: *const ArRtosStats) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    (*ptr).0.utilization
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_stats_schedulable(ptr: *const ArRtosStats) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    i32::from((*ptr).0.schedulable)
}

// ============================================================================
// Scheduler (19 functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_scheduler_new() -> *mut ArRtosScheduler {
    Box::into_raw(Box::new(ArRtosScheduler(Scheduler::new())))
}

/// # Safety
/// `ptr` must be a valid pointer from `ar_rtos_scheduler_new` or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_free(ptr: *mut ArRtosScheduler) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// `ptr` and `name_ptr` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_register(
    ptr: *mut ArRtosScheduler,
    name_ptr: *const u8,
    name_len: u32,
    priority: u8,
    period_us: u32,
    wcet_us: u32,
) -> i32 {
    if ptr.is_null() || name_ptr.is_null() {
        return -1;
    }
    let sched = &mut (*ptr).0;
    let name = core::slice::from_raw_parts(name_ptr, name_len as usize);
    let task = crate::task::Task::new(
        name,
        ffi_noop_task,
        TaskPriority(priority),
        period_us,
        wcet_us,
    );
    match sched.register(task) {
        Some(idx) => idx as i32,
        None => -1,
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_tick(ptr: *mut ArRtosScheduler, delta_us: u64) -> i32 {
    if ptr.is_null() {
        return -1;
    }
    match (*ptr).0.tick(delta_us) {
        Some(idx) => idx as i32,
        None => -1,
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_is_schedulable(ptr: *const ArRtosScheduler) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    i32::from((*ptr).0.is_schedulable())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_total_utilization(ptr: *const ArRtosScheduler) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    (*ptr).0.total_utilization()
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_active_task_count(ptr: *const ArRtosScheduler) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.active_task_count() as u32
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_now_us(ptr: *const ArRtosScheduler) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.now_us()
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_suspend(ptr: *mut ArRtosScheduler, idx: u32) {
    if !ptr.is_null() {
        (*ptr).0.suspend(idx as usize);
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_resume(ptr: *mut ArRtosScheduler, idx: u32) {
    if !ptr.is_null() {
        (*ptr).0.resume(idx as usize);
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_context_switches(ptr: *const ArRtosScheduler) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.context_switches
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_state(ptr: *const ArRtosScheduler, idx: u32) -> u8 {
    if ptr.is_null() {
        return 4;
    }
    match (*ptr).0.get_task(idx as usize) {
        Some(t) => match t.state {
            TaskState::Ready => 0,
            TaskState::Running => 1,
            TaskState::Sleeping => 2,
            TaskState::Suspended => 3,
            TaskState::Inactive => 4,
        },
        None => 4,
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_exec_count(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.get_task(idx as usize).map_or(0, |t| t.exec_count)
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_deadline_misses(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr)
        .0
        .get_task(idx as usize)
        .map_or(0, |t| t.deadline_misses)
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_utilization(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    (*ptr)
        .0
        .get_task(idx as usize)
        .map_or(0.0, |t| t.utilization())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_frequency(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    (*ptr)
        .0
        .get_task(idx as usize)
        .map_or(0.0, |t| t.frequency_hz())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_period(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.get_task(idx as usize).map_or(0, |t| t.period_us)
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_wcet(ptr: *const ArRtosScheduler, idx: u32) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.get_task(idx as usize).map_or(0, |t| t.wcet_us)
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_scheduler_task_priority(
    ptr: *const ArRtosScheduler,
    idx: u32,
) -> u8 {
    if ptr.is_null() {
        return 255;
    }
    (*ptr)
        .0
        .get_task(idx as usize)
        .map_or(255, |t| t.priority.0)
}

// ============================================================================
// Timer (11 functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_timer_new(clock_hz: u32) -> *mut ArRtosTimer {
    Box::into_raw(Box::new(ArRtosTimer(SysTimer::new(clock_hz))))
}

#[no_mangle]
pub extern "C" fn ar_rtos_timer_software() -> *mut ArRtosTimer {
    Box::into_raw(Box::new(ArRtosTimer(SysTimer::software())))
}

/// # Safety
/// `ptr` must be a valid pointer from `ar_rtos_timer_new` or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_free(ptr: *mut ArRtosTimer) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_advance(ptr: *mut ArRtosTimer, us: u64) {
    if !ptr.is_null() {
        (*ptr).0.advance(us);
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_now_us(ptr: *const ArRtosTimer) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.now_us()
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_now_ms(ptr: *const ArRtosTimer) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.now_ms()
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_now_secs(ptr: *const ArRtosTimer) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    (*ptr).0.now_secs()
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_reset(ptr: *mut ArRtosTimer) {
    if !ptr.is_null() {
        (*ptr).0.reset();
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_ticks_per_us(ptr: *const ArRtosTimer) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.ticks_per_us()
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_overflows(ptr: *const ArRtosTimer) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.overflows()
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_timer_elapsed_since(
    ptr: *const ArRtosTimer,
    reference: u64,
) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    (*ptr).0.elapsed_since(reference)
}

// ============================================================================
// SPSC Ring (9 functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_spsc_new() -> *mut ArRtosSpsc {
    Box::into_raw(Box::new(ArRtosSpsc(SpscRing::new())))
}

/// # Safety
/// `ptr` must be a valid pointer from `ar_rtos_spsc_new` or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_free(ptr: *mut ArRtosSpsc) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_push(ptr: *mut ArRtosSpsc, value: u32) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    i32::from((*ptr).0.push(value))
}

/// # Safety
/// `ptr` and `out_value` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_pop(ptr: *mut ArRtosSpsc, out_value: *mut u32) -> i32 {
    if ptr.is_null() || out_value.is_null() {
        return 0;
    }
    match (*ptr).0.pop() {
        Some(v) => {
            *out_value = v;
            1
        }
        None => 0,
    }
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_len(ptr: *const ArRtosSpsc) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    // SpscRing::len uses atomic loads (&self), safe with const ptr
    let ring = &(*ptr).0;
    ring.len() as u32
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_is_empty(ptr: *const ArRtosSpsc) -> i32 {
    if ptr.is_null() {
        return 1;
    }
    let ring = &(*ptr).0;
    i32::from(ring.is_empty())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_is_full(ptr: *const ArRtosSpsc) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    let ring = &(*ptr).0;
    i32::from(ring.is_full())
}

/// # Safety
/// `ptr` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_capacity(ptr: *const ArRtosSpsc) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let ring = &(*ptr).0;
    ring.capacity() as u32
}

/// # Safety
/// `ptr` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn ar_rtos_spsc_clear(ptr: *mut ArRtosSpsc) {
    if !ptr.is_null() {
        (*ptr).0.clear();
    }
}

// ============================================================================
// Deadline helpers (2 stateless functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_deadline_is_met(start: u64, period_us: u32, current: u64) -> i32 {
    let d = Deadline::new(start, period_us);
    i32::from(d.is_met(current))
}

#[no_mangle]
pub extern "C" fn ar_rtos_deadline_remaining(start: u64, period_us: u32, current: u64) -> u64 {
    let d = Deadline::new(start, period_us);
    d.remaining(current)
}

// ============================================================================
// Priority constants (5 functions)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_priority_critical() -> u8 {
    TaskPriority::CRITICAL.0
}

#[no_mangle]
pub extern "C" fn ar_rtos_priority_high() -> u8 {
    TaskPriority::HIGH.0
}

#[no_mangle]
pub extern "C" fn ar_rtos_priority_normal() -> u8 {
    TaskPriority::NORMAL.0
}

#[no_mangle]
pub extern "C" fn ar_rtos_priority_low() -> u8 {
    TaskPriority::LOW.0
}

#[no_mangle]
pub extern "C" fn ar_rtos_priority_idle() -> u8 {
    TaskPriority::IDLE.0
}

// ============================================================================
// Version (1 function)
// ============================================================================

#[no_mangle]
pub extern "C" fn ar_rtos_version() -> u32 {
    // 0.1.0 → 0x000100
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap_or(0);
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap_or(0);
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap_or(0);
    (major << 16) | (minor << 8) | patch
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_lifecycle() {
        let ptr = ar_rtos_kernel_new(72_000_000);
        assert!(!ptr.is_null());
        unsafe {
            let idx = ar_rtos_kernel_add_task(ptr, b"test".as_ptr(), 4, 2, 1000, 100);
            assert_eq!(idx, 0);
            assert_eq!(ar_rtos_kernel_is_schedulable(ptr), 1);
            assert_eq!(ar_rtos_kernel_active_task_count(ptr), 1);
            ar_rtos_kernel_free(ptr);
        }
    }

    #[test]
    fn test_kernel_tick() {
        let ptr = ar_rtos_kernel_testing();
        unsafe {
            ar_rtos_kernel_add_task(ptr, b"t1".as_ptr(), 2, 2, 100, 10);
            let executed = ar_rtos_kernel_tick(ptr, 0);
            assert_eq!(executed, 0);
            ar_rtos_kernel_free(ptr);
        }
    }

    #[test]
    fn test_kernel_run_for() {
        let ptr = ar_rtos_kernel_testing();
        unsafe {
            ar_rtos_kernel_add_task(ptr, b"fast".as_ptr(), 4, 1, 100, 10);
            let stats = ar_rtos_kernel_run_for(ptr, 1000, 10);
            assert!(!stats.is_null());
            assert!(ar_rtos_stats_tasks_executed(stats) > 0);
            assert_eq!(ar_rtos_stats_schedulable(stats), 1);
            ar_rtos_stats_free(stats);
            ar_rtos_kernel_free(ptr);
        }
    }

    #[test]
    fn test_scheduler_lifecycle() {
        let ptr = ar_rtos_scheduler_new();
        assert!(!ptr.is_null());
        unsafe {
            let idx = ar_rtos_scheduler_register(ptr, b"task1".as_ptr(), 5, 2, 100, 10);
            assert_eq!(idx, 0);
            assert_eq!(ar_rtos_scheduler_active_task_count(ptr), 1);
            assert!(ar_rtos_scheduler_total_utilization(ptr) > 0.0);
            ar_rtos_scheduler_free(ptr);
        }
    }

    #[test]
    fn test_scheduler_suspend_resume() {
        let ptr = ar_rtos_scheduler_new();
        unsafe {
            ar_rtos_scheduler_register(ptr, b"t".as_ptr(), 1, 2, 100, 10);
            ar_rtos_scheduler_suspend(ptr, 0);
            assert_eq!(ar_rtos_scheduler_task_state(ptr, 0), 3); // Suspended
            ar_rtos_scheduler_resume(ptr, 0);
            assert_eq!(ar_rtos_scheduler_task_state(ptr, 0), 0); // Ready
            ar_rtos_scheduler_free(ptr);
        }
    }

    #[test]
    fn test_timer_lifecycle() {
        let ptr = ar_rtos_timer_software();
        assert!(!ptr.is_null());
        unsafe {
            ar_rtos_timer_advance(ptr, 1000);
            assert_eq!(ar_rtos_timer_now_us(ptr), 1000);
            assert_eq!(ar_rtos_timer_now_ms(ptr), 1);
            ar_rtos_timer_reset(ptr);
            assert_eq!(ar_rtos_timer_now_us(ptr), 0);
            ar_rtos_timer_free(ptr);
        }
    }

    #[test]
    fn test_spsc_lifecycle() {
        let ptr = ar_rtos_spsc_new();
        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(ar_rtos_spsc_is_empty(ptr), 1);
            assert_eq!(ar_rtos_spsc_push(ptr, 42), 1);
            assert_eq!(ar_rtos_spsc_len(ptr), 1);
            let mut val: u32 = 0;
            assert_eq!(ar_rtos_spsc_pop(ptr, &mut val), 1);
            assert_eq!(val, 42);
            assert_eq!(ar_rtos_spsc_is_empty(ptr), 1);
            ar_rtos_spsc_free(ptr);
        }
    }

    #[test]
    fn test_deadline_helpers() {
        assert_eq!(ar_rtos_deadline_is_met(0, 1000, 500), 1);
        assert_eq!(ar_rtos_deadline_is_met(0, 1000, 1001), 0);
        assert_eq!(ar_rtos_deadline_remaining(0, 1000, 500), 500);
    }

    #[test]
    fn test_priority_constants() {
        assert_eq!(ar_rtos_priority_critical(), 0);
        assert_eq!(ar_rtos_priority_high(), 1);
        assert_eq!(ar_rtos_priority_normal(), 2);
        assert_eq!(ar_rtos_priority_low(), 3);
        assert_eq!(ar_rtos_priority_idle(), 255);
    }

    #[test]
    fn test_version() {
        let v = ar_rtos_version();
        assert_eq!(v, 0x000100); // 0.1.0
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            ar_rtos_kernel_free(core::ptr::null_mut());
            assert_eq!(ar_rtos_kernel_tick(core::ptr::null_mut(), 0), -1);
            assert_eq!(ar_rtos_kernel_is_schedulable(core::ptr::null()), 0);
            ar_rtos_scheduler_free(core::ptr::null_mut());
            ar_rtos_timer_free(core::ptr::null_mut());
            ar_rtos_spsc_free(core::ptr::null_mut());
            ar_rtos_stats_free(core::ptr::null_mut());
        }
    }
}
