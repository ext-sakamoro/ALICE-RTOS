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
//! | `edge` | no | ALICE-Edge model evaluation tasks |
//! | `synth` | no | ALICE-Synth audio render tasks |
//! | `motion` | no | ALICE-Motion trajectory tasks |
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

#![no_std]

pub mod kernel;
pub mod scheduler;
pub mod spsc;
pub mod task;
pub mod timer;

pub use kernel::Kernel;
pub use scheduler::Scheduler;
pub use spsc::SpscRing;
pub use task::{Task, TaskFn, TaskPriority, TaskState};
pub use timer::SysTimer;
