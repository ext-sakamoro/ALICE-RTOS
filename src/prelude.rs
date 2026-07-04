//! Convenience re-export (= `use alice_rtos::prelude::*;` で主要 API 一括取得)
//!
//! RTOS 6 core module (task / scheduler / timer / spsc / kernel / priority_inherit +
//! dmda 分析) から主要型 + 関数を re-export する
//! `edge_tasks` / `motion_tasks` / `synth_tasks` / `ffi` / `python` は
//! feature-gated

pub use crate::dmda::{analyze as dmda_analyze, DmdaReport, RtaResult};
pub use crate::kernel::{Kernel, KernelStats};
pub use crate::priority_inherit::{PipResult, PriorityInheritTracker, PriorityResource};
pub use crate::scheduler::Scheduler;
pub use crate::spsc::SpscRing;
pub use crate::task::{Task, TaskFn, TaskPriority, TaskState};
pub use crate::timer::{Deadline, SysTimer};
