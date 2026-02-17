//! ALICE-RTOS — Math-First Real-Time OS
//!
//! Don't schedule processes, schedule equations.
//!
//! Minimal RTOS kernel for deterministic equation evaluation:
//! - Static task table (no heap, no allocation)
//! - Rate-Monotonic Scheduling with deadline guarantees
//! - Zero-copy SPSC ring buffers for inter-task communication
//! - < 2 KB flash, < 256 B RAM for the scheduler
//!
//! Author: Moroya Sakamoto

#![no_std]

pub mod task;
pub mod scheduler;
pub mod timer;
pub mod spsc;
pub mod kernel;

pub use task::{Task, TaskState, TaskPriority, TaskFn};
pub use scheduler::Scheduler;
pub use timer::SysTimer;
pub use spsc::SpscRing;
pub use kernel::Kernel;
