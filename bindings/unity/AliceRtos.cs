// ALICE-RTOS Unity C# Bindings — 66 DllImport declarations + RAII handles
// Auto-generated from src/ffi.rs
// Copyright (C) 2026 Moroya Sakamoto — AGPL-3.0

using System;
using System.Runtime.InteropServices;

namespace Alice.Rtos
{
    // ========================================================================
    // Task state enum
    // ========================================================================

    public enum TaskState : byte
    {
        Ready = 0,
        Running = 1,
        Sleeping = 2,
        Suspended = 3,
        Inactive = 4,
    }

    // ========================================================================
    // RAII Handles
    // ========================================================================

    public class Kernel : IDisposable
    {
        internal IntPtr Ptr;
        private bool disposed;

        public Kernel(uint clockHz) { Ptr = AliceRtos.ar_rtos_kernel_new(clockHz); }
        private Kernel(IntPtr ptr) { Ptr = ptr; }

        public static Kernel Testing() => new Kernel(AliceRtos.ar_rtos_kernel_testing());

        public int AddTask(byte[] name, byte priority, uint periodUs, uint wcetUs)
        {
            unsafe
            {
                fixed (byte* p = name)
                {
                    return AliceRtos.ar_rtos_kernel_add_task(Ptr, (IntPtr)p, (uint)name.Length, priority, periodUs, wcetUs);
                }
            }
        }

        public int Tick(ulong deltaUs) => AliceRtos.ar_rtos_kernel_tick(Ptr, deltaUs);
        public KernelStats RunFor(ulong totalUs, ulong tickUs) => new KernelStats(AliceRtos.ar_rtos_kernel_run_for(Ptr, totalUs, tickUs));
        public void Stop() => AliceRtos.ar_rtos_kernel_stop(Ptr);
        public bool IsRunning => AliceRtos.ar_rtos_kernel_is_running(Ptr) != 0;
        public bool IsSchedulable => AliceRtos.ar_rtos_kernel_is_schedulable(Ptr) != 0;
        public uint MemoryFootprint => AliceRtos.ar_rtos_kernel_memory_footprint(Ptr);
        public ulong TotalTicks => AliceRtos.ar_rtos_kernel_total_ticks(Ptr);
        public uint ActiveTaskCount => AliceRtos.ar_rtos_kernel_active_task_count(Ptr);

        public void Dispose()
        {
            if (!disposed && Ptr != IntPtr.Zero) { AliceRtos.ar_rtos_kernel_free(Ptr); Ptr = IntPtr.Zero; disposed = true; }
            GC.SuppressFinalize(this);
        }
        ~Kernel() { Dispose(); }
    }

    public class KernelStats : IDisposable
    {
        internal IntPtr Ptr;
        private bool disposed;

        internal KernelStats(IntPtr ptr) { Ptr = ptr; }

        public ulong TotalUs => AliceRtos.ar_rtos_stats_total_us(Ptr);
        public ulong TotalTicks => AliceRtos.ar_rtos_stats_total_ticks(Ptr);
        public ulong TasksExecuted => AliceRtos.ar_rtos_stats_tasks_executed(Ptr);
        public ulong ContextSwitches => AliceRtos.ar_rtos_stats_context_switches(Ptr);
        public float Utilization => AliceRtos.ar_rtos_stats_utilization(Ptr);
        public bool Schedulable => AliceRtos.ar_rtos_stats_schedulable(Ptr) != 0;

        public void Dispose()
        {
            if (!disposed && Ptr != IntPtr.Zero) { AliceRtos.ar_rtos_stats_free(Ptr); Ptr = IntPtr.Zero; disposed = true; }
            GC.SuppressFinalize(this);
        }
        ~KernelStats() { Dispose(); }
    }

    public class Scheduler : IDisposable
    {
        internal IntPtr Ptr;
        private bool disposed;

        public Scheduler() { Ptr = AliceRtos.ar_rtos_scheduler_new(); }

        public int Register(byte[] name, byte priority, uint periodUs, uint wcetUs)
        {
            unsafe
            {
                fixed (byte* p = name)
                {
                    return AliceRtos.ar_rtos_scheduler_register(Ptr, (IntPtr)p, (uint)name.Length, priority, periodUs, wcetUs);
                }
            }
        }

        public int Tick(ulong deltaUs) => AliceRtos.ar_rtos_scheduler_tick(Ptr, deltaUs);
        public bool IsSchedulable => AliceRtos.ar_rtos_scheduler_is_schedulable(Ptr) != 0;
        public float TotalUtilization => AliceRtos.ar_rtos_scheduler_total_utilization(Ptr);
        public uint ActiveTaskCount => AliceRtos.ar_rtos_scheduler_active_task_count(Ptr);
        public ulong NowUs => AliceRtos.ar_rtos_scheduler_now_us(Ptr);
        public void Suspend(uint idx) => AliceRtos.ar_rtos_scheduler_suspend(Ptr, idx);
        public void Resume(uint idx) => AliceRtos.ar_rtos_scheduler_resume(Ptr, idx);
        public uint ContextSwitches => AliceRtos.ar_rtos_scheduler_context_switches(Ptr);

        public TaskState GetTaskState(uint idx) => (TaskState)AliceRtos.ar_rtos_scheduler_task_state(Ptr, idx);
        public uint GetTaskExecCount(uint idx) => AliceRtos.ar_rtos_scheduler_task_exec_count(Ptr, idx);
        public uint GetTaskDeadlineMisses(uint idx) => AliceRtos.ar_rtos_scheduler_task_deadline_misses(Ptr, idx);
        public float GetTaskUtilization(uint idx) => AliceRtos.ar_rtos_scheduler_task_utilization(Ptr, idx);
        public float GetTaskFrequency(uint idx) => AliceRtos.ar_rtos_scheduler_task_frequency(Ptr, idx);
        public uint GetTaskPeriod(uint idx) => AliceRtos.ar_rtos_scheduler_task_period(Ptr, idx);
        public uint GetTaskWcet(uint idx) => AliceRtos.ar_rtos_scheduler_task_wcet(Ptr, idx);
        public byte GetTaskPriority(uint idx) => AliceRtos.ar_rtos_scheduler_task_priority(Ptr, idx);

        public void Dispose()
        {
            if (!disposed && Ptr != IntPtr.Zero) { AliceRtos.ar_rtos_scheduler_free(Ptr); Ptr = IntPtr.Zero; disposed = true; }
            GC.SuppressFinalize(this);
        }
        ~Scheduler() { Dispose(); }
    }

    public class SysTimer : IDisposable
    {
        internal IntPtr Ptr;
        private bool disposed;

        public SysTimer(uint clockHz) { Ptr = AliceRtos.ar_rtos_timer_new(clockHz); }
        private SysTimer(IntPtr ptr) { Ptr = ptr; }

        public static SysTimer Software() => new SysTimer(AliceRtos.ar_rtos_timer_software());

        public void Advance(ulong us) => AliceRtos.ar_rtos_timer_advance(Ptr, us);
        public ulong NowUs => AliceRtos.ar_rtos_timer_now_us(Ptr);
        public ulong NowMs => AliceRtos.ar_rtos_timer_now_ms(Ptr);
        public float NowSecs => AliceRtos.ar_rtos_timer_now_secs(Ptr);
        public void Reset() => AliceRtos.ar_rtos_timer_reset(Ptr);
        public uint TicksPerUs => AliceRtos.ar_rtos_timer_ticks_per_us(Ptr);
        public uint Overflows => AliceRtos.ar_rtos_timer_overflows(Ptr);
        public ulong ElapsedSince(ulong reference) => AliceRtos.ar_rtos_timer_elapsed_since(Ptr, reference);

        public void Dispose()
        {
            if (!disposed && Ptr != IntPtr.Zero) { AliceRtos.ar_rtos_timer_free(Ptr); Ptr = IntPtr.Zero; disposed = true; }
            GC.SuppressFinalize(this);
        }
        ~SysTimer() { Dispose(); }
    }

    public class SpscRing : IDisposable
    {
        internal IntPtr Ptr;
        private bool disposed;

        public SpscRing() { Ptr = AliceRtos.ar_rtos_spsc_new(); }

        public bool Push(uint value) => AliceRtos.ar_rtos_spsc_push(Ptr, value) != 0;
        public bool Pop(out uint value)
        {
            value = 0;
            return AliceRtos.ar_rtos_spsc_pop(Ptr, out value) != 0;
        }
        public uint Length => AliceRtos.ar_rtos_spsc_len(Ptr);
        public bool IsEmpty => AliceRtos.ar_rtos_spsc_is_empty(Ptr) != 0;
        public bool IsFull => AliceRtos.ar_rtos_spsc_is_full(Ptr) != 0;
        public uint Capacity => AliceRtos.ar_rtos_spsc_capacity(Ptr);
        public void Clear() => AliceRtos.ar_rtos_spsc_clear(Ptr);

        public void Dispose()
        {
            if (!disposed && Ptr != IntPtr.Zero) { AliceRtos.ar_rtos_spsc_free(Ptr); Ptr = IntPtr.Zero; disposed = true; }
            GC.SuppressFinalize(this);
        }
        ~SpscRing() { Dispose(); }
    }

    // ========================================================================
    // DllImport declarations (66 functions)
    // ========================================================================

    public static class AliceRtos
    {
        private const string Lib = "alice_rtos";

        // Kernel (12)
        [DllImport(Lib)] public static extern IntPtr ar_rtos_kernel_new(uint clockHz);
        [DllImport(Lib)] public static extern IntPtr ar_rtos_kernel_testing();
        [DllImport(Lib)] public static extern void ar_rtos_kernel_free(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_kernel_add_task(IntPtr ptr, IntPtr namePtr, uint nameLen, byte priority, uint periodUs, uint wcetUs);
        [DllImport(Lib)] public static extern int ar_rtos_kernel_tick(IntPtr ptr, ulong deltaUs);
        [DllImport(Lib)] public static extern IntPtr ar_rtos_kernel_run_for(IntPtr ptr, ulong totalUs, ulong tickUs);
        [DllImport(Lib)] public static extern void ar_rtos_kernel_stop(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_kernel_is_running(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_kernel_is_schedulable(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_kernel_memory_footprint(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_kernel_total_ticks(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_kernel_active_task_count(IntPtr ptr);

        // KernelStats (7)
        [DllImport(Lib)] public static extern void ar_rtos_stats_free(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_stats_total_us(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_stats_total_ticks(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_stats_tasks_executed(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_stats_context_switches(IntPtr ptr);
        [DllImport(Lib)] public static extern float ar_rtos_stats_utilization(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_stats_schedulable(IntPtr ptr);

        // Scheduler (19)
        [DllImport(Lib)] public static extern IntPtr ar_rtos_scheduler_new();
        [DllImport(Lib)] public static extern void ar_rtos_scheduler_free(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_scheduler_register(IntPtr ptr, IntPtr namePtr, uint nameLen, byte priority, uint periodUs, uint wcetUs);
        [DllImport(Lib)] public static extern int ar_rtos_scheduler_tick(IntPtr ptr, ulong deltaUs);
        [DllImport(Lib)] public static extern int ar_rtos_scheduler_is_schedulable(IntPtr ptr);
        [DllImport(Lib)] public static extern float ar_rtos_scheduler_total_utilization(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_active_task_count(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_scheduler_now_us(IntPtr ptr);
        [DllImport(Lib)] public static extern void ar_rtos_scheduler_suspend(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern void ar_rtos_scheduler_resume(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_context_switches(IntPtr ptr);
        [DllImport(Lib)] public static extern byte ar_rtos_scheduler_task_state(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_task_exec_count(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_task_deadline_misses(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern float ar_rtos_scheduler_task_utilization(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern float ar_rtos_scheduler_task_frequency(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_task_period(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern uint ar_rtos_scheduler_task_wcet(IntPtr ptr, uint idx);
        [DllImport(Lib)] public static extern byte ar_rtos_scheduler_task_priority(IntPtr ptr, uint idx);

        // Timer (11)
        [DllImport(Lib)] public static extern IntPtr ar_rtos_timer_new(uint clockHz);
        [DllImport(Lib)] public static extern IntPtr ar_rtos_timer_software();
        [DllImport(Lib)] public static extern void ar_rtos_timer_free(IntPtr ptr);
        [DllImport(Lib)] public static extern void ar_rtos_timer_advance(IntPtr ptr, ulong us);
        [DllImport(Lib)] public static extern ulong ar_rtos_timer_now_us(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_timer_now_ms(IntPtr ptr);
        [DllImport(Lib)] public static extern float ar_rtos_timer_now_secs(IntPtr ptr);
        [DllImport(Lib)] public static extern void ar_rtos_timer_reset(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_timer_ticks_per_us(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_timer_overflows(IntPtr ptr);
        [DllImport(Lib)] public static extern ulong ar_rtos_timer_elapsed_since(IntPtr ptr, ulong reference);

        // SPSC Ring (9)
        [DllImport(Lib)] public static extern IntPtr ar_rtos_spsc_new();
        [DllImport(Lib)] public static extern void ar_rtos_spsc_free(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_spsc_push(IntPtr ptr, uint value);
        [DllImport(Lib)] public static extern int ar_rtos_spsc_pop(IntPtr ptr, out uint outValue);
        [DllImport(Lib)] public static extern uint ar_rtos_spsc_len(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_spsc_is_empty(IntPtr ptr);
        [DllImport(Lib)] public static extern int ar_rtos_spsc_is_full(IntPtr ptr);
        [DllImport(Lib)] public static extern uint ar_rtos_spsc_capacity(IntPtr ptr);
        [DllImport(Lib)] public static extern void ar_rtos_spsc_clear(IntPtr ptr);

        // Deadline helpers (2)
        [DllImport(Lib)] public static extern int ar_rtos_deadline_is_met(ulong start, uint periodUs, ulong current);
        [DllImport(Lib)] public static extern ulong ar_rtos_deadline_remaining(ulong start, uint periodUs, ulong current);

        // Priority constants (5)
        [DllImport(Lib)] public static extern byte ar_rtos_priority_critical();
        [DllImport(Lib)] public static extern byte ar_rtos_priority_high();
        [DllImport(Lib)] public static extern byte ar_rtos_priority_normal();
        [DllImport(Lib)] public static extern byte ar_rtos_priority_low();
        [DllImport(Lib)] public static extern byte ar_rtos_priority_idle();

        // Version (1)
        [DllImport(Lib)] public static extern uint ar_rtos_version();
    }
}
