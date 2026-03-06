//! Deadline Miss Detectability Analysis (DMDA)
//!
//! Response-Time Analysis (RTA) によるタスクごとの最悪応答時間を計算し、
//! デッドライン充足を検証する。Liu & Layland bound より正確。
//!
//! Author: Moroya Sakamoto

use crate::task::{Task, MAX_TASKS};

/// RTA 結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtaResult {
    /// タスクインデックス。
    pub task_index: usize,
    /// 最悪応答時間（µs）。
    pub worst_case_response: u32,
    /// デッドライン（= 周期、µs）。
    pub deadline: u32,
    /// デッドライン充足判定。
    pub meets_deadline: bool,
}

/// DMDA 分析結果。
#[derive(Debug, Clone)]
pub struct DmdaReport {
    /// タスクごとの RTA 結果。
    pub results: [Option<RtaResult>; MAX_TASKS],
    /// 分析対象タスク数。
    pub task_count: usize,
    /// 全タスクがデッドラインを満たすか。
    pub all_schedulable: bool,
}

/// Response-Time Analysis を実行する。
///
/// 各タスク `i` の最悪応答時間 `R_i` を反復計算:
///   `R_i(n+1) = C_i + Σ(j ∈ hp(i)) ⌈R_i(n) / T_j⌉ × C_j`
///
/// ここで `hp(i)` はタスク `i` より高優先度のタスク集合。
/// `R_i ≤ T_i`（デッドライン = 周期）なら充足。
///
/// `tasks` は優先度順（低い priority 値 = 高優先度が先頭）にソートされている前提。
/// `task_count` は有効タスク数。
#[must_use]
pub fn analyze(tasks: &[Task; MAX_TASKS], task_count: usize) -> DmdaReport {
    let mut report = DmdaReport {
        results: [None; MAX_TASKS],
        task_count,
        all_schedulable: true,
    };

    // 有効タスクを優先度順に収集（priority 値が小さい = 高優先度）
    let mut sorted_indices: [usize; MAX_TASKS] = [0; MAX_TASKS];
    let mut count = 0;
    for (i, task) in tasks.iter().enumerate().take(task_count) {
        if task.is_active() {
            sorted_indices[count] = i;
            count += 1;
        }
    }

    // 挿入ソート（優先度順）
    for i in 1..count {
        let key = sorted_indices[i];
        let mut j = i;
        while j > 0 && tasks[sorted_indices[j - 1]].priority > tasks[key].priority {
            sorted_indices[j] = sorted_indices[j - 1];
            j -= 1;
        }
        sorted_indices[j] = key;
    }

    // 各タスクの RTA
    for rank in 0..count {
        let idx = sorted_indices[rank];
        let task = &tasks[idx];
        let wcet = task.wcet_us;
        let period = task.period_us;

        if period == 0 {
            continue;
        }

        // 反復計算: R = C_i + Σ hp ⌈R / T_j⌉ × C_j
        let mut r = wcet;
        let max_iterations = 100_u32;

        for _ in 0..max_iterations {
            let mut interference: u64 = 0;

            // 高優先度タスク（rank より前）からの干渉
            for &hp_idx in &sorted_indices[..rank] {
                let hp = &tasks[hp_idx];
                if hp.period_us == 0 {
                    continue;
                }
                // ⌈R / T_j⌉ × C_j
                let preemptions = u64::from(r).div_ceil(u64::from(hp.period_us));
                interference += preemptions * u64::from(hp.wcet_us);
            }

            let new_r = u64::from(wcet) + interference;
            let new_r = if new_r > u64::from(u32::MAX) {
                u32::MAX
            } else {
                new_r as u32
            };

            if new_r == r {
                // 収束
                break;
            }

            // デッドライン超過で早期打ち切り
            if new_r > period {
                r = new_r;
                break;
            }

            r = new_r;
        }

        let meets = r <= period;
        if !meets {
            report.all_schedulable = false;
        }

        report.results[idx] = Some(RtaResult {
            task_index: idx,
            worst_case_response: r,
            deadline: period,
            meets_deadline: meets,
        });
    }

    report
}

/// 最も危険な（デッドラインマージンが最小の）タスクを返す。
#[must_use]
pub fn most_critical_task(report: &DmdaReport) -> Option<RtaResult> {
    let mut worst: Option<RtaResult> = None;
    let mut min_margin = i64::MAX;

    for r in report.results.iter().flatten() {
        let margin = i64::from(r.deadline) - i64::from(r.worst_case_response);
        if margin < min_margin {
            min_margin = margin;
            worst = Some(*r);
        }
    }

    worst
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskPriority};

    fn dummy(_: &mut [u8]) {}

    fn make_tasks(specs: &[(TaskPriority, u32, u32)]) -> ([Task; MAX_TASKS], usize) {
        let mut tasks = [Task::empty(); MAX_TASKS];
        for (i, &(prio, period, wcet)) in specs.iter().enumerate() {
            tasks[i] = Task::new(b"t", dummy, prio, period, wcet);
        }
        (tasks, specs.len())
    }

    #[test]
    fn single_task_meets_deadline() {
        let (tasks, count) = make_tasks(&[(TaskPriority::CRITICAL, 100, 10)]);
        let report = analyze(&tasks, count);
        assert!(report.all_schedulable);
        let r = report.results[0].unwrap();
        assert_eq!(r.worst_case_response, 10);
        assert!(r.meets_deadline);
    }

    #[test]
    fn single_task_exceeds_deadline() {
        let (tasks, count) = make_tasks(&[(TaskPriority::CRITICAL, 100, 110)]);
        let report = analyze(&tasks, count);
        assert!(!report.all_schedulable);
        assert!(!report.results[0].unwrap().meets_deadline);
    }

    #[test]
    fn two_tasks_schedulable() {
        // タスク0: CRITICAL, period=100, wcet=20
        // タスク1: NORMAL,   period=200, wcet=50
        // R1 = 50 + ⌈R1/100⌉×20
        //    = 50 + 1×20 = 70 ≤ 200 ✓
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 100, 20),
            (TaskPriority::NORMAL, 200, 50),
        ]);
        let report = analyze(&tasks, count);
        assert!(report.all_schedulable);
    }

    #[test]
    fn two_tasks_not_schedulable() {
        // タスク0: CRITICAL, period=100, wcet=60
        // タスク1: NORMAL,   period=100, wcet=60
        // R1 = 60 + ⌈R1/100⌉×60 → 60+60=120 > 100 ✗
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 100, 60),
            (TaskPriority::NORMAL, 100, 60),
        ]);
        let report = analyze(&tasks, count);
        assert!(!report.all_schedulable);
    }

    #[test]
    fn three_tasks_interference() {
        // T0: CRITICAL, period=50,  wcet=10
        // T1: HIGH,     period=100, wcet=20
        // T2: NORMAL,   period=200, wcet=40
        // R0 = 10 ≤ 50 ✓
        // R1 = 20 + ⌈20/50⌉×10 = 20+10=30 ≤ 100 ✓
        // R2: 40→70(⌈40/50⌉×10+⌈40/100⌉×20=30)→80(⌈70/50⌉×10+⌈70/100⌉×20=40)→80 収束 ≤ 200 ✓
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 50, 10),
            (TaskPriority::HIGH, 100, 20),
            (TaskPriority::NORMAL, 200, 40),
        ]);
        let report = analyze(&tasks, count);
        assert!(report.all_schedulable);
        assert_eq!(report.results[0].unwrap().worst_case_response, 10);
        assert_eq!(report.results[1].unwrap().worst_case_response, 30);
        assert_eq!(report.results[2].unwrap().worst_case_response, 80);
    }

    #[test]
    fn empty_tasks() {
        let tasks = [Task::empty(); MAX_TASKS];
        let report = analyze(&tasks, 0);
        assert!(report.all_schedulable);
        assert_eq!(report.task_count, 0);
    }

    #[test]
    fn most_critical_finds_tightest() {
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 50, 10),
            (TaskPriority::HIGH, 100, 20),
            (TaskPriority::NORMAL, 200, 40),
        ]);
        let report = analyze(&tasks, count);
        let critical = most_critical_task(&report).unwrap();
        // マージン: T0=40, T1=70, T2=130 → T0が最小
        assert_eq!(critical.task_index, 0);
    }

    #[test]
    fn most_critical_empty() {
        let tasks = [Task::empty(); MAX_TASKS];
        let report = analyze(&tasks, 0);
        assert!(most_critical_task(&report).is_none());
    }

    #[test]
    fn rta_result_fields() {
        let r = RtaResult {
            task_index: 3,
            worst_case_response: 50,
            deadline: 100,
            meets_deadline: true,
        };
        assert_eq!(r.task_index, 3);
        assert_eq!(r.worst_case_response, 50);
        assert_eq!(r.deadline, 100);
        assert!(r.meets_deadline);
    }

    #[test]
    fn report_clone() {
        let (tasks, count) = make_tasks(&[(TaskPriority::CRITICAL, 100, 10)]);
        let report = analyze(&tasks, count);
        let cloned = report.clone();
        assert_eq!(cloned.all_schedulable, report.all_schedulable);
        assert_eq!(cloned.task_count, report.task_count);
    }

    #[test]
    fn wcet_equals_period_exact_boundary() {
        // wcet == period → R = period → meets (ちょうど境界)
        let (tasks, count) = make_tasks(&[(TaskPriority::CRITICAL, 100, 100)]);
        let report = analyze(&tasks, count);
        assert!(report.all_schedulable);
        assert!(report.results[0].unwrap().meets_deadline);
    }

    #[test]
    fn high_interference_misses() {
        // 高優先度タスクの干渉で低優先度がデッドライン超過
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 20, 10), // 50% CPU
            (TaskPriority::NORMAL, 30, 15),   // R = 15 + ⌈15/20⌉×10 = 25 ≤ 30 → R=25+10=35? 再計算
        ]);
        let report = analyze(&tasks, count);
        // R1: 初期 R=15, 干渉=⌈15/20⌉×10=10, new_R=25
        //     R=25, 干渉=⌈25/20⌉×10=20, new_R=35 > 30 → miss
        assert!(!report.results[1].unwrap().meets_deadline);
    }

    #[test]
    fn most_critical_with_miss() {
        let (tasks, count) = make_tasks(&[
            (TaskPriority::CRITICAL, 20, 10),
            (TaskPriority::NORMAL, 30, 15),
        ]);
        let report = analyze(&tasks, count);
        let critical = most_critical_task(&report).unwrap();
        // T1 がデッドライン超過 → マージン負
        assert_eq!(critical.task_index, 1);
    }
}
