//! 優先度継承プロトコル (Priority Inheritance Protocol)
//!
//! 優先度逆転を防ぐリソースロック機構。
//! 低優先度タスクがリソースを保持中に高優先度タスクがブロックされた場合、
//! 低優先度タスクの優先度を一時的に引き上げる。
//!
//! Author: Moroya Sakamoto

use crate::task::{TaskPriority, MAX_TASKS};

/// 最大リソース数。
pub const MAX_RESOURCES: usize = 8;

/// 優先度継承リソース。
///
/// タスク間で共有されるリソース（ミューテックス相当）。
/// ロック保持中のタスクに対して優先度継承を適用する。
#[derive(Debug, Clone, Copy)]
pub struct PriorityResource {
    /// リソース名（8 ASCII 文字まで）。
    pub name: [u8; 8],
    /// 現在ロックしているタスクのインデックス（`None` = 空き）。
    pub holder: Option<usize>,
    /// ロック保持者の元の優先度（復元用）。
    pub original_priority: TaskPriority,
    /// 待機中タスクの最高優先度（継承先）。
    pub ceiling: TaskPriority,
}

impl PriorityResource {
    /// 空リソース。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            name: [0u8; 8],
            holder: None,
            original_priority: TaskPriority::IDLE,
            ceiling: TaskPriority::IDLE,
        }
    }

    /// 名前付きリソース作成。
    #[must_use]
    pub fn new(name: &[u8]) -> Self {
        let mut n = [0u8; 8];
        let len = name.len().min(8);
        n[..len].copy_from_slice(&name[..len]);
        Self {
            name: n,
            holder: None,
            original_priority: TaskPriority::IDLE,
            ceiling: TaskPriority::IDLE,
        }
    }

    /// リソースが使用中か。
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.holder.is_some()
    }
}

/// 優先度継承トラッカー。
///
/// リソースの獲得・解放と優先度の一時的な引き上げ・復元を管理する。
pub struct PriorityInheritTracker {
    /// リソーステーブル。
    resources: [PriorityResource; MAX_RESOURCES],
    /// 登録済みリソース数。
    resource_count: usize,
    /// タスクごとの有効優先度（継承後）。
    effective_priorities: [TaskPriority; MAX_TASKS],
    /// タスクごとの基本優先度。
    base_priorities: [TaskPriority; MAX_TASKS],
}

/// 優先度継承操作の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipResult {
    /// 成功。
    Ok,
    /// リソースが既に別タスクに保持されている。
    Blocked {
        holder: usize,
        inherited_priority: TaskPriority,
    },
    /// リソースが見つからない。
    ResourceNotFound,
    /// リソーステーブルが満杯。
    TableFull,
    /// このタスクはリソースを保持していない。
    NotHolder,
}

impl PriorityInheritTracker {
    /// 新規作成。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            resources: [PriorityResource::empty(); MAX_RESOURCES],
            resource_count: 0,
            effective_priorities: [TaskPriority::IDLE; MAX_TASKS],
            base_priorities: [TaskPriority::IDLE; MAX_TASKS],
        }
    }

    /// タスクの基本優先度を登録。
    pub const fn register_task(&mut self, task_idx: usize, priority: TaskPriority) {
        if task_idx < MAX_TASKS {
            self.base_priorities[task_idx] = priority;
            self.effective_priorities[task_idx] = priority;
        }
    }

    /// リソースを登録。
    pub fn register_resource(&mut self, name: &[u8]) -> PipResult {
        if self.resource_count >= MAX_RESOURCES {
            return PipResult::TableFull;
        }
        self.resources[self.resource_count] = PriorityResource::new(name);
        self.resource_count += 1;
        PipResult::Ok
    }

    /// リソースを獲得する。
    ///
    /// 空きなら即座にロック。保持中なら `Blocked` を返し、
    /// 保持者の優先度を要求者の優先度まで引き上げる（継承）。
    pub fn acquire(&mut self, resource_idx: usize, task_idx: usize) -> PipResult {
        if resource_idx >= self.resource_count {
            return PipResult::ResourceNotFound;
        }

        let res = &mut self.resources[resource_idx];

        if let Some(holder) = res.holder {
            if holder == task_idx {
                // 再入: 既に保持している
                return PipResult::Ok;
            }

            // ブロック → 優先度継承
            let requester_prio = self.effective_priorities[task_idx];

            // ceiling を常に最高待機優先度で更新
            if requester_prio < res.ceiling {
                res.ceiling = requester_prio;
            }

            if requester_prio < self.effective_priorities[holder] {
                // 要求者が高優先度 → 保持者の有効優先度を引き上げ
                self.effective_priorities[holder] = requester_prio;
            }

            PipResult::Blocked {
                holder,
                inherited_priority: self.effective_priorities[holder],
            }
        } else {
            // 空き → ロック獲得
            res.holder = Some(task_idx);
            res.original_priority = self.base_priorities[task_idx];
            res.ceiling = self.effective_priorities[task_idx];
            PipResult::Ok
        }
    }

    /// リソースを解放する。
    ///
    /// 保持者の優先度を基本優先度に復元する。
    pub fn release(&mut self, resource_idx: usize, task_idx: usize) -> PipResult {
        if resource_idx >= self.resource_count {
            return PipResult::ResourceNotFound;
        }

        if self.resources[resource_idx].holder != Some(task_idx) {
            return PipResult::NotHolder;
        }

        // リソースを解放
        self.resources[resource_idx].holder = None;
        self.resources[resource_idx].ceiling = TaskPriority::IDLE;

        // 優先度を基本に復元
        self.effective_priorities[task_idx] = self.base_priorities[task_idx];

        // 他のリソースを保持中なら、その中で最高の ceiling を維持
        for i in 0..self.resource_count {
            if self.resources[i].holder == Some(task_idx) {
                let ceil = self.resources[i].ceiling;
                if ceil < self.effective_priorities[task_idx] {
                    self.effective_priorities[task_idx] = ceil;
                }
            }
        }

        PipResult::Ok
    }

    /// タスクの有効優先度を取得。
    #[must_use]
    pub const fn effective_priority(&self, task_idx: usize) -> TaskPriority {
        if task_idx < MAX_TASKS {
            self.effective_priorities[task_idx]
        } else {
            TaskPriority::IDLE
        }
    }

    /// タスクの基本優先度を取得。
    #[must_use]
    pub const fn base_priority(&self, task_idx: usize) -> TaskPriority {
        if task_idx < MAX_TASKS {
            self.base_priorities[task_idx]
        } else {
            TaskPriority::IDLE
        }
    }

    /// リソース参照。
    #[must_use]
    pub const fn get_resource(&self, idx: usize) -> Option<&PriorityResource> {
        if idx < self.resource_count {
            Some(&self.resources[idx])
        } else {
            None
        }
    }

    /// 登録済みリソース数。
    #[must_use]
    pub const fn resource_count(&self) -> usize {
        self.resource_count
    }
}

impl Default for PriorityInheritTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker_empty() {
        let tracker = PriorityInheritTracker::new();
        assert_eq!(tracker.resource_count(), 0);
    }

    #[test]
    fn register_resource() {
        let mut tracker = PriorityInheritTracker::new();
        assert_eq!(tracker.register_resource(b"mutex1"), PipResult::Ok);
        assert_eq!(tracker.resource_count(), 1);
    }

    #[test]
    fn register_max_resources() {
        let mut tracker = PriorityInheritTracker::new();
        for i in 0..MAX_RESOURCES {
            let name = [b'r', b'0' + i as u8];
            assert_eq!(tracker.register_resource(&name), PipResult::Ok);
        }
        assert_eq!(tracker.register_resource(b"over"), PipResult::TableFull);
    }

    #[test]
    fn acquire_free_resource() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::NORMAL);
        tracker.register_resource(b"m");
        assert_eq!(tracker.acquire(0, 0), PipResult::Ok);
        assert!(tracker.get_resource(0).unwrap().is_locked());
    }

    #[test]
    fn acquire_invalid_resource() {
        let mut tracker = PriorityInheritTracker::new();
        assert_eq!(tracker.acquire(99, 0), PipResult::ResourceNotFound);
    }

    #[test]
    fn release_resource() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::NORMAL);
        tracker.register_resource(b"m");
        tracker.acquire(0, 0);
        assert_eq!(tracker.release(0, 0), PipResult::Ok);
        assert!(!tracker.get_resource(0).unwrap().is_locked());
    }

    #[test]
    fn release_not_holder() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::NORMAL);
        tracker.register_task(1, TaskPriority::HIGH);
        tracker.register_resource(b"m");
        tracker.acquire(0, 0);
        assert_eq!(tracker.release(0, 1), PipResult::NotHolder);
    }

    #[test]
    fn priority_inheritance_on_block() {
        let mut tracker = PriorityInheritTracker::new();
        // タスク0: LOW (低優先度) がリソースを保持
        // タスク1: CRITICAL (高優先度) がリソースを要求 → ブロック → タスク0の優先度がCRITICALに
        tracker.register_task(0, TaskPriority::LOW);
        tracker.register_task(1, TaskPriority::CRITICAL);
        tracker.register_resource(b"m");

        tracker.acquire(0, 0);
        assert_eq!(tracker.effective_priority(0), TaskPriority::LOW);

        let result = tracker.acquire(0, 1);
        assert!(matches!(result, PipResult::Blocked { holder: 0, .. }));
        // タスク0 の有効優先度が CRITICAL に引き上げられた
        assert_eq!(tracker.effective_priority(0), TaskPriority::CRITICAL);
    }

    #[test]
    fn priority_restored_on_release() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::LOW);
        tracker.register_task(1, TaskPriority::CRITICAL);
        tracker.register_resource(b"m");

        tracker.acquire(0, 0);
        tracker.acquire(0, 1); // blocked → inherit
        assert_eq!(tracker.effective_priority(0), TaskPriority::CRITICAL);

        tracker.release(0, 0);
        // 基本優先度に復元
        assert_eq!(tracker.effective_priority(0), TaskPriority::LOW);
    }

    #[test]
    fn reentrant_acquire() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::NORMAL);
        tracker.register_resource(b"m");
        tracker.acquire(0, 0);
        // 同一タスクが再度 acquire → Ok（再入）
        assert_eq!(tracker.acquire(0, 0), PipResult::Ok);
    }

    #[test]
    fn multiple_resources_priority_maintained() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::LOW);
        tracker.register_task(1, TaskPriority::CRITICAL);
        tracker.register_task(2, TaskPriority::HIGH);
        tracker.register_resource(b"m1");
        tracker.register_resource(b"m2");

        // タスク0 が m1, m2 を保持
        tracker.acquire(0, 0);
        tracker.acquire(1, 0);
        // タスク1(CRITICAL) が m1 を要求 → 継承
        tracker.acquire(0, 1);
        assert_eq!(tracker.effective_priority(0), TaskPriority::CRITICAL);

        // タスク2(HIGH) が m2 を要求 → 継承（ただし CRITICAL > HIGH なので変化なし）
        tracker.acquire(1, 2);
        assert_eq!(tracker.effective_priority(0), TaskPriority::CRITICAL);

        // m1 を解放 → m2 でまだ HIGH の ceiling がある
        tracker.release(0, 0);
        assert_eq!(tracker.effective_priority(0), TaskPriority::HIGH);

        // m2 を解放 → 基本優先度に復元
        tracker.release(1, 0);
        assert_eq!(tracker.effective_priority(0), TaskPriority::LOW);
    }

    #[test]
    fn base_priority_unchanged() {
        let mut tracker = PriorityInheritTracker::new();
        tracker.register_task(0, TaskPriority::LOW);
        tracker.register_task(1, TaskPriority::CRITICAL);
        tracker.register_resource(b"m");

        tracker.acquire(0, 0);
        tracker.acquire(0, 1);
        // base は変わらない
        assert_eq!(tracker.base_priority(0), TaskPriority::LOW);
        assert_eq!(tracker.effective_priority(0), TaskPriority::CRITICAL);
    }

    #[test]
    fn effective_priority_out_of_bounds() {
        let tracker = PriorityInheritTracker::new();
        assert_eq!(
            tracker.effective_priority(MAX_TASKS + 1),
            TaskPriority::IDLE
        );
    }

    #[test]
    fn default_tracker() {
        let tracker = PriorityInheritTracker::default();
        assert_eq!(tracker.resource_count(), 0);
    }

    #[test]
    fn resource_empty() {
        let r = PriorityResource::empty();
        assert!(!r.is_locked());
        assert_eq!(r.holder, None);
    }

    #[test]
    fn resource_new_name() {
        let r = PriorityResource::new(b"mutex_a");
        assert_eq!(&r.name[..7], b"mutex_a");
        assert!(!r.is_locked());
    }

    #[test]
    fn resource_name_truncation() {
        let r = PriorityResource::new(b"very_long_name");
        assert_eq!(&r.name, b"very_lon");
    }

    #[test]
    fn release_invalid_resource() {
        let mut tracker = PriorityInheritTracker::new();
        assert_eq!(tracker.release(99, 0), PipResult::ResourceNotFound);
    }

    #[test]
    fn get_resource_out_of_bounds() {
        let tracker = PriorityInheritTracker::new();
        assert!(tracker.get_resource(0).is_none());
    }

    #[test]
    fn pip_result_eq() {
        assert_eq!(PipResult::Ok, PipResult::Ok);
        assert_eq!(PipResult::TableFull, PipResult::TableFull);
        assert_ne!(PipResult::Ok, PipResult::NotHolder);
    }
}
