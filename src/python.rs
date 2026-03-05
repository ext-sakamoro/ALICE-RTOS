//! PyO3 Python bindings for ALICE-RTOS
//!
//! Provides Python classes for RTOS scheduling simulation and monitoring.
//!
//! Author: Moroya Sakamoto

use pyo3::prelude::*;

use crate::kernel::Kernel as RustKernel;
use crate::scheduler::Scheduler as RustScheduler;
use crate::spsc::SpscRing;
use crate::task::TaskPriority;
use crate::timer::SysTimer as RustSysTimer;

/// ALICE-RTOS Kernel for scheduling simulation
#[pyclass(name = "Kernel")]
pub struct PyKernel {
    inner: RustKernel,
}

#[pymethods]
impl PyKernel {
    #[new]
    #[pyo3(signature = (clock_hz=72_000_000))]
    fn new(clock_hz: u32) -> Self {
        Self {
            inner: RustKernel::new(clock_hz),
        }
    }

    /// Create a kernel for testing (software timer)
    #[staticmethod]
    fn testing() -> Self {
        Self {
            inner: RustKernel::testing(),
        }
    }

    /// Add a task (uses no-op function for simulation)
    #[pyo3(signature = (name, priority, period_us, wcet_us))]
    fn add_task(
        &mut self,
        name: &str,
        priority: u8,
        period_us: u32,
        wcet_us: u32,
    ) -> Option<usize> {
        fn noop(_: &mut [u8]) {}
        self.inner.add_task(
            name.as_bytes(),
            noop,
            TaskPriority(priority),
            period_us,
            wcet_us,
        )
    }

    /// Advance one tick, returns task index or None
    fn tick(&mut self, delta_us: u64) -> Option<usize> {
        self.inner.tick(delta_us)
    }

    /// Run for a duration, returns stats dict
    fn run_for(&mut self, total_us: u64, tick_us: u64) -> PyKernelStats {
        let stats = self.inner.run_for(total_us, tick_us);
        PyKernelStats { inner: stats }
    }

    /// Stop the kernel
    fn stop(&mut self) {
        self.inner.stop();
    }

    #[getter]
    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    #[getter]
    fn is_schedulable(&self) -> bool {
        self.inner.is_schedulable()
    }

    #[getter]
    fn memory_footprint(&self) -> usize {
        self.inner.memory_footprint()
    }

    #[getter]
    fn total_ticks(&self) -> u64 {
        self.inner.total_ticks
    }

    #[getter]
    fn active_task_count(&self) -> usize {
        self.inner.scheduler.active_task_count()
    }

    #[getter]
    fn utilization(&self) -> f32 {
        self.inner.scheduler.total_utilization()
    }
}

/// Kernel execution statistics
#[pyclass(name = "KernelStats")]
pub struct PyKernelStats {
    inner: crate::kernel::KernelStats,
}

#[pymethods]
impl PyKernelStats {
    #[getter]
    fn total_us(&self) -> u64 {
        self.inner.total_us
    }

    #[getter]
    fn total_ticks(&self) -> u64 {
        self.inner.total_ticks
    }

    #[getter]
    fn tasks_executed(&self) -> u64 {
        self.inner.tasks_executed
    }

    #[getter]
    fn context_switches(&self) -> u64 {
        self.inner.context_switches
    }

    #[getter]
    fn utilization(&self) -> f32 {
        self.inner.utilization
    }

    #[getter]
    fn schedulable(&self) -> bool {
        self.inner.schedulable
    }

    fn __repr__(&self) -> String {
        format!(
            "KernelStats(tasks_executed={}, ctx_switches={}, utilization={:.3}, schedulable={})",
            self.inner.tasks_executed,
            self.inner.context_switches,
            self.inner.utilization,
            self.inner.schedulable
        )
    }
}

/// Rate-Monotonic Scheduler
#[pyclass(name = "Scheduler")]
pub struct PyScheduler {
    inner: RustScheduler,
}

#[pymethods]
impl PyScheduler {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustScheduler::new(),
        }
    }

    /// Register a task, returns slot index
    fn register(
        &mut self,
        name: &str,
        priority: u8,
        period_us: u32,
        wcet_us: u32,
    ) -> Option<usize> {
        fn noop(_: &mut [u8]) {}
        let task = crate::task::Task::new(
            name.as_bytes(),
            noop,
            TaskPriority(priority),
            period_us,
            wcet_us,
        );
        self.inner.register(task)
    }

    /// Advance one tick
    fn tick(&mut self, delta_us: u64) -> Option<usize> {
        self.inner.tick(delta_us)
    }

    fn suspend(&mut self, idx: usize) {
        self.inner.suspend(idx);
    }

    fn resume(&mut self, idx: usize) {
        self.inner.resume(idx);
    }

    #[getter]
    fn is_schedulable(&self) -> bool {
        self.inner.is_schedulable()
    }

    #[getter]
    fn total_utilization(&self) -> f32 {
        self.inner.total_utilization()
    }

    #[getter]
    fn active_task_count(&self) -> usize {
        self.inner.active_task_count()
    }

    #[getter]
    fn now_us(&self) -> u64 {
        self.inner.now_us()
    }

    #[getter]
    fn context_switches(&self) -> u32 {
        self.inner.context_switches
    }
}

/// System timer
#[pyclass(name = "SysTimer")]
pub struct PySysTimer {
    inner: RustSysTimer,
}

#[pymethods]
impl PySysTimer {
    #[new]
    #[pyo3(signature = (clock_hz=72_000_000))]
    fn new(clock_hz: u32) -> Self {
        Self {
            inner: RustSysTimer::new(clock_hz),
        }
    }

    #[staticmethod]
    fn software() -> Self {
        Self {
            inner: RustSysTimer::software(),
        }
    }

    fn advance(&mut self, us: u64) {
        self.inner.advance(us);
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    #[getter]
    fn now_us(&self) -> u64 {
        self.inner.now_us()
    }

    #[getter]
    fn now_ms(&self) -> u64 {
        self.inner.now_ms()
    }

    #[getter]
    fn now_secs(&self) -> f32 {
        self.inner.now_secs()
    }

    #[getter]
    fn overflows(&self) -> u32 {
        self.inner.overflows()
    }
}

/// Lock-free SPSC ring buffer (256 slots)
#[pyclass(name = "SpscRing")]
pub struct PySpscRing {
    inner: SpscRing<256>,
}

#[pymethods]
impl PySpscRing {
    #[new]
    fn new() -> Self {
        Self {
            inner: SpscRing::new(),
        }
    }

    fn push(&mut self, value: u32) -> bool {
        self.inner.push(value)
    }

    fn pop(&mut self) -> Option<u32> {
        self.inner.pop()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    #[getter]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[getter]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[getter]
    fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    #[getter]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

/// Register ALICE-RTOS Python module
#[pymodule]
fn alice_rtos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyKernel>()?;
    m.add_class::<PyKernelStats>()?;
    m.add_class::<PyScheduler>()?;
    m.add_class::<PySysTimer>()?;
    m.add_class::<PySpscRing>()?;
    Ok(())
}
