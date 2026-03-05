// ALICE-RTOS UE5 C++ Bindings — 66 extern "C" declarations + RAII handles
// Auto-generated from src/ffi.rs
// Copyright (C) 2026 Moroya Sakamoto — AGPL-3.0

#pragma once

#include <cstdint>
#include <memory>

// ============================================================================
// Opaque handle forward declarations
// ============================================================================

struct ArRtosKernel;
struct ArRtosScheduler;
struct ArRtosTimer;
struct ArRtosSpsc;
struct ArRtosStats;

// ============================================================================
// C-ABI declarations (66 functions)
// ============================================================================

extern "C"
{
    // Kernel (12)
    ArRtosKernel* ar_rtos_kernel_new(uint32_t clockHz);
    ArRtosKernel* ar_rtos_kernel_testing();
    void ar_rtos_kernel_free(ArRtosKernel* ptr);
    int32_t ar_rtos_kernel_add_task(ArRtosKernel* ptr, const uint8_t* namePtr, uint32_t nameLen, uint8_t priority, uint32_t periodUs, uint32_t wcetUs);
    int32_t ar_rtos_kernel_tick(ArRtosKernel* ptr, uint64_t deltaUs);
    ArRtosStats* ar_rtos_kernel_run_for(ArRtosKernel* ptr, uint64_t totalUs, uint64_t tickUs);
    void ar_rtos_kernel_stop(ArRtosKernel* ptr);
    int32_t ar_rtos_kernel_is_running(const ArRtosKernel* ptr);
    int32_t ar_rtos_kernel_is_schedulable(const ArRtosKernel* ptr);
    uint32_t ar_rtos_kernel_memory_footprint(const ArRtosKernel* ptr);
    uint64_t ar_rtos_kernel_total_ticks(const ArRtosKernel* ptr);
    uint32_t ar_rtos_kernel_active_task_count(const ArRtosKernel* ptr);

    // KernelStats (7)
    void ar_rtos_stats_free(ArRtosStats* ptr);
    uint64_t ar_rtos_stats_total_us(const ArRtosStats* ptr);
    uint64_t ar_rtos_stats_total_ticks(const ArRtosStats* ptr);
    uint64_t ar_rtos_stats_tasks_executed(const ArRtosStats* ptr);
    uint64_t ar_rtos_stats_context_switches(const ArRtosStats* ptr);
    float ar_rtos_stats_utilization(const ArRtosStats* ptr);
    int32_t ar_rtos_stats_schedulable(const ArRtosStats* ptr);

    // Scheduler (19)
    ArRtosScheduler* ar_rtos_scheduler_new();
    void ar_rtos_scheduler_free(ArRtosScheduler* ptr);
    int32_t ar_rtos_scheduler_register(ArRtosScheduler* ptr, const uint8_t* namePtr, uint32_t nameLen, uint8_t priority, uint32_t periodUs, uint32_t wcetUs);
    int32_t ar_rtos_scheduler_tick(ArRtosScheduler* ptr, uint64_t deltaUs);
    int32_t ar_rtos_scheduler_is_schedulable(const ArRtosScheduler* ptr);
    float ar_rtos_scheduler_total_utilization(const ArRtosScheduler* ptr);
    uint32_t ar_rtos_scheduler_active_task_count(const ArRtosScheduler* ptr);
    uint64_t ar_rtos_scheduler_now_us(const ArRtosScheduler* ptr);
    void ar_rtos_scheduler_suspend(ArRtosScheduler* ptr, uint32_t idx);
    void ar_rtos_scheduler_resume(ArRtosScheduler* ptr, uint32_t idx);
    uint32_t ar_rtos_scheduler_context_switches(const ArRtosScheduler* ptr);
    uint8_t ar_rtos_scheduler_task_state(const ArRtosScheduler* ptr, uint32_t idx);
    uint32_t ar_rtos_scheduler_task_exec_count(const ArRtosScheduler* ptr, uint32_t idx);
    uint32_t ar_rtos_scheduler_task_deadline_misses(const ArRtosScheduler* ptr, uint32_t idx);
    float ar_rtos_scheduler_task_utilization(const ArRtosScheduler* ptr, uint32_t idx);
    float ar_rtos_scheduler_task_frequency(const ArRtosScheduler* ptr, uint32_t idx);
    uint32_t ar_rtos_scheduler_task_period(const ArRtosScheduler* ptr, uint32_t idx);
    uint32_t ar_rtos_scheduler_task_wcet(const ArRtosScheduler* ptr, uint32_t idx);
    uint8_t ar_rtos_scheduler_task_priority(const ArRtosScheduler* ptr, uint32_t idx);

    // Timer (11)
    ArRtosTimer* ar_rtos_timer_new(uint32_t clockHz);
    ArRtosTimer* ar_rtos_timer_software();
    void ar_rtos_timer_free(ArRtosTimer* ptr);
    void ar_rtos_timer_advance(ArRtosTimer* ptr, uint64_t us);
    uint64_t ar_rtos_timer_now_us(const ArRtosTimer* ptr);
    uint64_t ar_rtos_timer_now_ms(const ArRtosTimer* ptr);
    float ar_rtos_timer_now_secs(const ArRtosTimer* ptr);
    void ar_rtos_timer_reset(ArRtosTimer* ptr);
    uint32_t ar_rtos_timer_ticks_per_us(const ArRtosTimer* ptr);
    uint32_t ar_rtos_timer_overflows(const ArRtosTimer* ptr);
    uint64_t ar_rtos_timer_elapsed_since(const ArRtosTimer* ptr, uint64_t reference);

    // SPSC Ring (9)
    ArRtosSpsc* ar_rtos_spsc_new();
    void ar_rtos_spsc_free(ArRtosSpsc* ptr);
    int32_t ar_rtos_spsc_push(ArRtosSpsc* ptr, uint32_t value);
    int32_t ar_rtos_spsc_pop(ArRtosSpsc* ptr, uint32_t* outValue);
    uint32_t ar_rtos_spsc_len(const ArRtosSpsc* ptr);
    int32_t ar_rtos_spsc_is_empty(const ArRtosSpsc* ptr);
    int32_t ar_rtos_spsc_is_full(const ArRtosSpsc* ptr);
    uint32_t ar_rtos_spsc_capacity(const ArRtosSpsc* ptr);
    void ar_rtos_spsc_clear(ArRtosSpsc* ptr);

    // Deadline helpers (2)
    int32_t ar_rtos_deadline_is_met(uint64_t start, uint32_t periodUs, uint64_t current);
    uint64_t ar_rtos_deadline_remaining(uint64_t start, uint32_t periodUs, uint64_t current);

    // Priority constants (5)
    uint8_t ar_rtos_priority_critical();
    uint8_t ar_rtos_priority_high();
    uint8_t ar_rtos_priority_normal();
    uint8_t ar_rtos_priority_low();
    uint8_t ar_rtos_priority_idle();

    // Version (1)
    uint32_t ar_rtos_version();
}

// ============================================================================
// RAII Handle Wrappers
// ============================================================================

namespace Alice { namespace Rtos {

struct KernelDeleter { void operator()(ArRtosKernel* p) const { ar_rtos_kernel_free(p); } };
using KernelHandle = std::unique_ptr<ArRtosKernel, KernelDeleter>;

struct StatsDeleter { void operator()(ArRtosStats* p) const { ar_rtos_stats_free(p); } };
using StatsHandle = std::unique_ptr<ArRtosStats, StatsDeleter>;

struct SchedulerDeleter { void operator()(ArRtosScheduler* p) const { ar_rtos_scheduler_free(p); } };
using SchedulerHandle = std::unique_ptr<ArRtosScheduler, SchedulerDeleter>;

struct TimerDeleter { void operator()(ArRtosTimer* p) const { ar_rtos_timer_free(p); } };
using TimerHandle = std::unique_ptr<ArRtosTimer, TimerDeleter>;

struct SpscDeleter { void operator()(ArRtosSpsc* p) const { ar_rtos_spsc_free(p); } };
using SpscHandle = std::unique_ptr<ArRtosSpsc, SpscDeleter>;

}} // namespace Alice::Rtos
