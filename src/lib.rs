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
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`task`] | Static no-alloc task descriptors with priority and WCET |
//! | [`scheduler`] | Rate-Monotonic scheduler with deadline tracking |
//! | [`timer`] | Hardware-abstracted system timer (tick / µs / ms) |
//! | [`spsc`] | Lock-free single-producer single-consumer ring buffer |
//! | [`kernel`] | Top-level kernel combining scheduler + timer + scratch |
//!
//! # Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `cortex-m` | no | ARM Cortex-M (M0/M4/M7) target support |
//! | `riscv` | no | RISC-V (ESP32-C3, GD32VF103) target support |
//! | `esp32` | no | Xtensa ESP32/ESP32-S3 target support |
//! | `edge` | no | ALICE-Edge task templates (1 kHz inference) |
//! | `synth` | no | ALICE-Synth task templates (44.1 kHz audio) |
//! | `motion` | no | ALICE-Motion task templates (10 kHz trajectory) |
//! | `ffi` | no | C-ABI FFI for Unity/UE5 (66 functions) |
//! | `python` | no | `PyO3` Python bindings |
//!
//! # Quick Start
//!
//! ```rust
//! use alice_rtos::{Kernel, TaskPriority};
//!
//! fn my_task(scratch: &mut [u8]) {
//!     scratch[0] = 42;
//! }
//!
//! let mut kernel = Kernel::new(72_000_000); // 72 MHz clock
//! let idx = kernel.add_task(b"eq1", my_task, TaskPriority::NORMAL, 1_000, 100);
//! assert!(idx.is_some());
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::module_name_repetitions,
    clippy::inline_always,
    clippy::too_many_lines
)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod dmda;
#[cfg(feature = "edge")]
pub mod edge_tasks;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod kernel;
#[cfg(feature = "motion")]
pub mod motion_tasks;
pub mod priority_inherit;
#[cfg(feature = "python")]
mod python;
pub mod scheduler;
pub mod spsc;
#[cfg(feature = "synth")]
pub mod synth_tasks;
pub mod task;
pub mod timer;

pub use dmda::{analyze as dmda_analyze, DmdaReport, RtaResult};
pub use kernel::{Kernel, KernelStats};
pub use priority_inherit::{PipResult, PriorityInheritTracker, PriorityResource};
pub use scheduler::Scheduler;
pub use spsc::SpscRing;
pub use task::{Task, TaskFn, TaskPriority, TaskState};
pub use timer::{Deadline, SysTimer};
